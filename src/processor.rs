// src/processor.rs
use crate::converters::{convert_docx_to_txt, convert_rtf_to_txt};
use anyhow::{Context, Result};
use log::{error, info};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_LINES_PER_FILE: usize = 50000;

/// Phase 1: Convert DOCX/RTF files from input_dir to TXT in temp_dir while preserving structure.
pub fn convert_files(input_dir: &Path, temp_dir: &Path) -> Result<usize> {
    info!("Starting conversion phase: Input='{}', Temp='{}'", input_dir.display(), temp_dir.display());
    let mut count = 0;

    // Traverse the input directory recursively
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

        // Select the appropriate converter
        let content_result = match extension.as_str() {
            "docx" => convert_docx_to_txt(path),
            "rtf" => convert_rtf_to_txt(path),
            _ => continue,
        };

        let content = match content_result {
            Ok(c) => c,
            Err(e) => {
                // Log the error but continue processing other files
                error!("Failed to convert {:?}: {}", path, e);
                continue;
            }
        };

        // Calculate the destination path while preserving relative structure
        let relative_path = path.strip_prefix(input_dir)?;
        let mut dest_path = temp_dir.join(relative_path);
        dest_path.set_extension("txt");

        info!("Converted: {:?}", relative_path);

        // Ensure the destination subdirectory exists in the temp directory
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(dest_path, content)?;
        count += 1;
    }

    Ok(count)
}

/// Phase 2: Concatenate TXT files from temp_dir into output files in output_dir with a line limit.
pub fn concatenate_files(temp_dir: &Path, output_dir: &Path) -> Result<()> {
    info!("Starting concatenation phase: Output='{}'", output_dir.display());

    let mut result_index = 1;
    let mut current_line_count = 0;
    let mut output_file = create_output_file(output_dir, result_index)?;

    // Collect and sort TXT files to guarantee deterministic output order
    let mut txt_files: Vec<PathBuf> = WalkDir::new(temp_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("txt"))
        .collect();

    txt_files.sort();

    for txt_path in txt_files {
        info!("Aggregating: {:?}", txt_path.strip_prefix(temp_dir).unwrap_or(&txt_path));
        let file = File::open(&txt_path)?;
        let mut reader = BufReader::new(file);

        // Read the full file content for processing
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        
        // Apply structured formatting for better LLM/RAG compatibility
        let formatted_content = format_medical_report(&content);
        
        // Process the complete content to avoid splitting reports mid-way
        let lines: Vec<&str> = formatted_content.lines().collect();
        let mut i = 0;
        let mut first_report_in_file = current_line_count == 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Check if this line marks the beginning of a new report
            let is_new_report = line.starts_with("TÉCNICA DO EXAME") || 
                               line.starts_with("INFORME CLÍNICO") ||
                               line.starts_with("RESSONÂNCIA") ||
                               line.starts_with("TOMOGRAFIA") ||
                               line.starts_with("ULTRASSONOGRAFIA") ||
                               line.starts_with("RAIO-X") ||
                               line.starts_with("MAMOGRAFIA") ||
                               line.starts_with("DENSITOMETRIA");
            
            // Add spacing between reports (except for the first report in the file)
            if is_new_report && !first_report_in_file && current_line_count > 0 {
                writeln!(output_file, "")?;
                writeln!(output_file, "")?;
                current_line_count += 2;
            }
            
            // Mark that we are no longer on the first report
            if is_new_report {
                first_report_in_file = false;
            }
            
            // If we are near the limit and this line starts a new report,
            // roll over BEFORE starting the new report
            if current_line_count >= MAX_LINES_PER_FILE - 100 && is_new_report && current_line_count > 0 {
                result_index += 1;
                output_file = create_output_file(output_dir, result_index)?;
                current_line_count = 0;
                first_report_in_file = true; // Reset for the new file
                info!("Rolling over to result_{}.txt (before new report)", result_index);
            }
            
            // If we still exceed the limit, roll over immediately
            if current_line_count >= MAX_LINES_PER_FILE {
                result_index += 1;
                output_file = create_output_file(output_dir, result_index)?;
                current_line_count = 0;
                first_report_in_file = true; // Reset for the new file
                info!("Rolling over to result_{}.txt (forced)", result_index);
            }
            
            writeln!(output_file, "{}", line)?;
            current_line_count += 1;
            i += 1;
        }
    }

    info!("Concatenation finished. Total files created: {}", result_index);
    Ok(())
}

/// Helper function to create a new (truncated) result file.
fn create_output_file(output_dir: &Path, index: usize) -> Result<File> {
    let path = output_dir.join(format!("result_{}.txt", index));
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Ensure the file starts empty
        .open(path)
        .context("Failed to create output result file")
}

/// Formats medical reports into a structure optimized for LLM/RAG
/// Preserves original line breaks and clarifies section structure
fn format_medical_report(content: &str) -> String {
    let content = content.trim();
    
    // Preserve original line breaks, only normalize extra spaces
    let lines: Vec<String> = content.lines()
        .map(|line| {
            // Remove extra spaces within each line
            line.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect();
    
    // Remove consecutive empty lines (maximum 1 empty line)
    let mut result_lines: Vec<String> = Vec::new();
    let mut last_was_empty = false;
    
    for line in lines.iter() {
        let is_empty = line.trim().is_empty();
        if is_empty && last_was_empty {
            continue; // Skip consecutive empty lines
        }
        result_lines.push(line.clone());
        last_was_empty = is_empty;
    }
    
    let mut result = result_lines.join("\n");
    
    // Add spacing after punctuation that is stuck to text
    result = fix_missing_spaces(&result);
    
    // Normalize section variants to a standard format
    let normalizations = [
        ("TÉCNICA DO EXAME:", "Técnica do exame:"),
        ("TÉCNICA DE EXAME:", "Técnica de exame:"),
        ("ASPECTOS OBSERVADOS:", "Aspectos observados:"),
        ("IMPRESSÃO DIAGNÓSTICA:", "IMPRESSÃO:"),
        ("Impressão:", "IMPRESSÃO:"),
        ("INFORME CLÍNICO:", "Informe clínico:"),
    ];
    
    for (from, to) in normalizations.iter() {
        result = result.replace(from, to);
    }
    
    // Ensure a line break BEFORE main sections
    let section_markers = [
        "Técnica do exame:",
        "Técnica de exame:",
        "Aspectos observados:",
        "IMPRESSÃO:",
        "Informe clínico:",
    ];
    
    for marker in section_markers.iter() {
        // If the marker is not at the start, add a break before it
        if !result.starts_with(marker) {
            // Add two line breaks before the marker (if not already present)
            let pattern_with_break = format!("\n\n{}", marker);
            let pattern_with_single_break = format!("\n{}", marker);
            
            if !result.contains(&pattern_with_break) {
                if result.contains(&pattern_with_single_break) {
                    result = result.replace(&pattern_with_single_break, &pattern_with_break);
                } else {
                    result = result.replace(marker, &pattern_with_break);
                }
            }
        }
        
        // Add a break AFTER the marker if text is attached to it
        let marker_space = format!("{} ", marker);
        let marker_newline = format!("{}\n", marker);
        if result.contains(&marker_space) && !result.contains(&marker_newline) {
            result = result.replace(&marker_space, &marker_newline);
        }
    }
    
    // Remove multiple consecutive line breaks (maximum 2)
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    
    // Remove a leading line break if present
    result = result.trim_start_matches('\n').to_string();
    
    result.trim().to_string()
}

/// Fixes missing spaces after punctuation
fn fix_missing_spaces(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    
    for i in 0..chars.len() {
        result.push(chars[i]);
        
        // If a colon is followed by an uppercase letter with no space, add a space
        // Except if the next letter is lowercase (could be part of a URL, time, etc.)
        if i + 1 < chars.len() {
            let current = chars[i];
            let next = chars[i + 1];
            
            // Add space after a colon followed by an uppercase letter
            if current == ':' && next.is_alphabetic() && next.is_uppercase() {
                result.push(' ');
            }
            // Add space after a period followed by an uppercase letter (new sentence)
            if current == '.' && next.is_alphabetic() && next.is_uppercase() {
                result.push(' ');
            }
        }
    }
    
    result
}
