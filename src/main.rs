// src/main.rs
mod converters;
mod processor;

use anyhow::{Context, Result};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
enum SetupStatus {
    Ready,
    CreatedInputDir,
}

fn main() -> Result<()> {
    // Initialize the logger with output to both terminal and file
    let log_file = File::create("log.txt").context("Failed to create log file")?;

    CombinedLogger::init(vec![
        // Terminal: INFO and above
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // File: WARN and ERROR (for diagnostics)
        WriteLogger::new(LevelFilter::Warn, Config::default(), log_file),
    ])
    .context("Failed to initialize logger")?;

    let input_dir = PathBuf::from("./docs");
    let temp_dir = PathBuf::from("./temp");
    let output_dir = PathBuf::from("./output");

    // 1. Initial setup and cleanup
    let setup_status = match setup(&input_dir, &output_dir) {
        Ok(status) => status,
        Err(e) => {
            log::error!("Setup failed: {:?}", e);
            return Err(e);
        }
    };

    if setup_status == SetupStatus::CreatedInputDir {
        return Ok(());
    }

    // Ensure the temporary directory is clean before starting
    if let Err(e) = cleanup(&temp_dir) {
        log::error!("Initial cleanup failed: {:?}", e);
        return Err(e);
    }
    fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

    // 2. Phase 1: Conversion (DOCX/RTF -> TXT)
    let count = match processor::convert_files(&input_dir, &temp_dir) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error during conversion phase: {:?}", e);
            cleanup(&temp_dir).ok(); // Attempt cleanup even if conversion fails
            return Err(e);
        }
    };

    // 3. Phase 2: Concatenation (TXT -> result_x.txt)
    if count > 0 {
        log::info!("Conversion phase completed. {} files converted.", count);
        if let Err(e) = processor::concatenate_files(&temp_dir, &output_dir) {
            log::error!("Error during concatenation phase: {:?}", e);
            cleanup(&temp_dir).ok(); // Attempt cleanup even if concatenation fails
            return Err(e);
        }
    } else {
        log::info!("No .docx or .rtf files found or converted in ./docs.");
    }

    // 4. Final cleanup
    cleanup(&temp_dir)?;

    log::info!("Processing finished successfully. Results are in the './output' directory.");
    Ok(())
}

/// Prepare input and output directories.
fn setup(input_dir: &Path, output_dir: &Path) -> Result<SetupStatus> {
    // Create the input directory if it doesn't exist
    if !input_dir.exists() {
        fs::create_dir_all(input_dir)?;
        log::info!(
            "Input directory '{}' created. Please add files and run again.",
            input_dir.display()
        );
        return Ok(SetupStatus::CreatedInputDir);
    }

    // Ensure the output directory exists and clean previous result files
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        // Remove only files matching the result_*.txt pattern
        if path.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("txt")
            && path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with("result_")
        {
            fs::remove_file(path)?;
        }
    }

    Ok(SetupStatus::Ready)
}

#[cfg(test)]
mod tests {
    use super::{cleanup, setup, SetupStatus};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "reports-to-llm-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn setup_creates_missing_input_dir_without_exiting() {
        let base_dir = unique_test_dir("setup-create-input");
        let input_dir = base_dir.join("docs");
        let output_dir = base_dir.join("output");

        let status =
            setup(&input_dir, &output_dir).expect("setup should create the input directory");

        assert_eq!(status, SetupStatus::CreatedInputDir);
        assert!(input_dir.exists());
        assert!(
            !output_dir.exists(),
            "output cleanup should not run on early return"
        );

        if base_dir.exists() {
            fs::remove_dir_all(&base_dir).expect("temporary test directory should be removable");
        }
    }

    #[test]
    fn setup_removes_only_previous_result_files() {
        let base_dir = unique_test_dir("setup-clean-output");
        let input_dir = base_dir.join("docs");
        let output_dir = base_dir.join("output");
        fs::create_dir_all(&input_dir).expect("input dir should be created");
        fs::create_dir_all(&output_dir).expect("output dir should be created");

        let stale_result = output_dir.join("result_1.txt");
        let preserved_non_txt_result = output_dir.join("result_1.csv");
        let preserved_file = output_dir.join("notes.txt");
        fs::write(&stale_result, "old result").expect("stale result file should be writable");
        fs::write(&preserved_non_txt_result, "keep me")
            .expect("non-txt result file should be writable");
        fs::write(&preserved_file, "keep me").expect("non-result file should be writable");

        let status = setup(&input_dir, &output_dir).expect("setup should clean existing results");

        assert_eq!(status, SetupStatus::Ready);
        assert!(
            !stale_result.exists(),
            "stale result files should be removed"
        );
        assert!(
            preserved_file.exists(),
            "non-result files should be preserved"
        );
        assert!(
            preserved_non_txt_result.exists(),
            "result-prefixed files without .txt extension should be preserved"
        );

        if base_dir.exists() {
            fs::remove_dir_all(&base_dir).expect("temporary test directory should be removable");
        }
    }

    #[test]
    fn cleanup_removes_existing_temp_dir() {
        let temp_dir = unique_test_dir("cleanup-temp-dir");
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        fs::write(temp_dir.join("temp.txt"), "temporary").expect("temp file should be writable");

        cleanup(&temp_dir).expect("cleanup should remove the temp directory");

        assert!(!temp_dir.exists());
    }
}

/// Clean the temporary directory.
fn cleanup(temp_dir: &Path) -> Result<()> {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).context("Failed to clean temp directory")?;
        log::info!("Temporary directory cleaned up.");
    }
    Ok(())
}
