// tests/output_quality.rs
// Quality tests to verify consistency of generated outputs

use std::fs;
use std::path::Path;

/// Reads all result files from the output directory
fn read_all_output_files() -> Vec<(String, String)> {
    let output_dir = Path::new("./output");
    let mut files = Vec::new();
    
    if !output_dir.exists() {
        return files;
    }
    
    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "txt") {
                if let Ok(content) = fs::read_to_string(&path) {
                    files.push((path.display().to_string(), content));
                }
            }
        }
    }
    
    files
}

/// Checks whether leaked RTF commands remain in the text
fn has_rtf_commands(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    let rtf_patterns = [
        ("s20", "RTF font size command"),
        ("fs20", "RTF font size command"),
        ("fs22", "RTF font size command"),
        ("s16", "RTF font size command"),
        ("fs16", "RTF font size command"),
        ("hfdbch", "RTF configuration command"),
        ("mdefJc", "RTF configuration command"),
        ("~b0", "RTF control character"),
        ("~b ", "RTF control character"),
        ("eflang1046", "RTF language command"),
        ("deflang1046", "RTF language command"),
    ];
    
    for (pattern, description) in rtf_patterns.iter() {
        if content.contains(pattern) {
            issues.push(format!("Found '{}': {}", pattern, description));
        }
    }
    
    issues
}

/// Checks for RTF encoding errors (non-decoded characters)
fn has_encoding_errors(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Patterns of undecoded RTF characters
    let encoding_patterns = [
        ("'e3", "ã not decoded"),
        ("'e9", "é not decoded"),
        ("'f3", "ó not decoded"),
        ("'e7", "ç not decoded"),
        ("'ed", "í not decoded"),
        ("'e1", "á not decoded"),
        ("'f5", "õ not decoded"),
        ("'e2", "â not decoded"),
        ("'ea", "ê not decoded"),
        ("'fa", "ú not decoded"),
    ];
    
    for (pattern, description) in encoding_patterns.iter() {
        if content.contains(pattern) {
            issues.push(format!("Found '{}': {}", pattern, description));
        }
    }
    
    issues
}

/// Checks for leaked font metadata
fn has_font_metadata(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    let font_patterns = [
        "Arial-BoldMT",
        "ArialMT",
        "Arial-ItalicMT",
        "Msftedit",
        ";;;*",
    ];
    
    for pattern in font_patterns.iter() {
        if content.contains(pattern) {
            issues.push(format!("Font metadata found: '{}'", pattern));
        }
    }
    
    // Check lines that appear to be font metadata (multiple fonts separated by ;)
    for line in content.lines() {
        if line.contains(';') && 
           (line.contains("Arial") || line.contains("Calibri") || line.contains("Helvetica")) &&
           line.matches(';').count() >= 2 {
            issues.push(format!("Font metadata line: '{}'", 
                               &line[..std::cmp::min(50, line.len())]));
            break; // Report only once
        }
    }
    
    issues
}

/// Checks for leaked XML tags (from DOCX files)
fn has_xml_tags(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    let xml_patterns = [
        ("<w:t>", "DOCX text XML tag"),
        ("</w:t>", "DOCX text XML tag"),
        ("<w:p>", "DOCX paragraph XML tag"),
        ("<w:tab", "DOCX tabulation XML tag"),
        ("<w:br", "DOCX line break XML tag"),
    ];
    
    for (pattern, description) in xml_patterns.iter() {
        if content.contains(pattern) {
            issues.push(format!("Found '{}': {}", pattern, description));
        }
    }
    
    issues
}

/// Checks whether reports have the expected basic structure
fn check_report_structure(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Ensure there is at least some medical content
    let expected_sections = [
        "Técnica do exame",
        "Aspectos observados",
        "IMPRESSÃO",
    ];
    
    let mut found_sections = 0;
    for section in expected_sections.iter() {
        if content.to_lowercase().contains(&section.to_lowercase()) {
            found_sections += 1;
        }
    }
    
    if found_sections == 0 {
        issues.push("No standard report section found".to_string());
    }
    
    // Ensure there are no excessively long lines (possible missing breaks)
    for (i, line) in content.lines().enumerate() {
        if line.len() > 5000 {
            issues.push(format!("Line {} excessively long ({} chars) - possible formatting issue", 
                               i + 1, line.len()));
        }
    }
    
    issues
}

/// Checks whether spacing between reports is adequate
fn check_report_spacing(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Check if there is at least one blank line anywhere
    // (indicating separation between sections/reports)
    if !content.contains("\n\n") {
        issues.push("No blank lines found - possible spacing issue".to_string());
    }
    
    issues
}

// ============================================================================
// TESTES
// ============================================================================

#[test]
fn test_no_rtf_commands_in_output() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = has_rtf_commands(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    assert!(
        all_issues.is_empty(),
        "\n❌ RTF commands found in outputs:\n{}\n",
        all_issues.join("\n")
    );
    
    println!("✅ No leaked RTF commands found");
}

#[test]
fn test_no_encoding_errors_in_output() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = has_encoding_errors(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    assert!(
        all_issues.is_empty(),
        "\n❌ Encoding errors found in outputs:\n{}\n",
        all_issues.join("\n")
    );
    
    println!("✅ No encoding errors found");
}

#[test]
fn test_no_font_metadata_in_output() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = has_font_metadata(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    assert!(
        all_issues.is_empty(),
        "\n❌ Font metadata found in outputs:\n{}\n",
        all_issues.join("\n")
    );
    
    println!("✅ No leaked font metadata found");
}

#[test]
fn test_no_xml_tags_in_output() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = has_xml_tags(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    assert!(
        all_issues.is_empty(),
        "\n❌ XML tags found in outputs:\n{}\n",
        all_issues.join("\n")
    );
    
    println!("✅ No leaked XML tags found");
}

#[test]
fn test_report_structure() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = check_report_structure(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    // This test is informational; it does not fail
    if !all_issues.is_empty() {
        println!("⚠️  Possible structure issues:\n{}", all_issues.join("\n"));
    } else {
        println!("✅ Report structure OK");
    }
}

#[test]
fn test_report_spacing() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut all_issues = Vec::new();
    
    for (filename, content) in &files {
        let issues = check_report_spacing(content);
        if !issues.is_empty() {
            all_issues.push(format!("{}:\n  - {}", filename, issues.join("\n  - ")));
        }
    }
    
    assert!(
        all_issues.is_empty(),
        "\n❌ Spacing problems found:\n{}\n",
        all_issues.join("\n")
    );
    
    println!("✅ Spacing between reports OK");
}

#[test]
fn test_output_files_exist() {
    let files = read_all_output_files();
    
    assert!(
        !files.is_empty(),
        "❌ No output files found in ./output/. Run 'cargo run' first."
    );
    
    println!("✅ {} output file(s) found", files.len());
}

#[test]
fn test_output_files_not_empty() {
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found. Run 'cargo run' first.");
        return;
    }
    
    let mut empty_files = Vec::new();
    
    for (filename, content) in &files {
        if content.trim().is_empty() {
            empty_files.push(filename.clone());
        }
    }
    
    assert!(
        empty_files.is_empty(),
        "\n❌ Empty files found:\n{}\n",
        empty_files.join("\n")
    );
    
    println!("✅ All output files contain content");
}

// ============================================================================
// COMPLETE TEST (runs all checks)
// ============================================================================

#[test]
fn test_full_output_quality() {
    println!("\n========================================");
    println!("  OUTPUT QUALITY VERIFICATION");
    println!("========================================\n");
    
    let files = read_all_output_files();
    
    if files.is_empty() {
        println!("⚠️  No output files found.");
        println!("   Run 'cargo run' first to generate outputs.\n");
        return;
    }
    
    println!("📁 {} file(s) found\n", files.len());
    
    let mut total_issues = 0;
    
    for (filename, content) in &files {
        let mut file_issues = Vec::new();
        
        file_issues.extend(has_rtf_commands(content));
        file_issues.extend(has_encoding_errors(content));
        file_issues.extend(has_font_metadata(content));
        file_issues.extend(has_xml_tags(content));
        file_issues.extend(check_report_spacing(content));
        
        if file_issues.is_empty() {
            println!("✅ {}", filename);
        } else {
            println!("❌ {} ({} issue(s))", filename, file_issues.len());
            for issue in &file_issues {
                println!("   - {}", issue);
            }
            total_issues += file_issues.len();
        }
    }
    
    println!("\n========================================");
    if total_issues == 0 {
        println!("  ✅ ALL TESTS PASSED!");
    } else {
        println!("  ❌ {} ISSUE(S) FOUND", total_issues);
    }
    println!("========================================\n");
    
    assert_eq!(total_issues, 0, "Quality issues found in outputs");
}
