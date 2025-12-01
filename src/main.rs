// src/main.rs
mod converters;
mod processor;

use anyhow::{Context, Result};
use simplelog::{
    CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger, ColorChoice,
};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

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
    if let Err(e) = setup(&input_dir, &output_dir) {
        log::error!("Setup failed: {:?}", e);
        return Err(e);
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
fn setup(input_dir: &PathBuf, output_dir: &PathBuf) -> Result<()> {
    // Create the input directory if it doesn't exist
    if !input_dir.exists() {
        fs::create_dir_all(input_dir)?;
        log::info!("Input directory './docs' created. Please add files and run again.");
        // Exit gracefully if the input directory was just created (it's empty)
        std::process::exit(0);
    }
    
    // Ensure the output directory exists and clean previous result files
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        // Remove only files matching the result_*.txt pattern
        if path.is_file() && path.file_name().unwrap_or_default().to_string_lossy().starts_with("result_") {
            fs::remove_file(path)?;
        }
    }
    
    Ok(())
}

/// Clean the temporary directory.
fn cleanup(temp_dir: &Path) -> Result<()> {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).context("Failed to clean temp directory")?;
        log::info!("Temporary directory cleaned up.");
    }
    Ok(())
}
