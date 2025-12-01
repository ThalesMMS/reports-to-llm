// src/converters.rs
use anyhow::{Context, Result};
use docx_rust::DocxFile;
use std::fs;
use std::path::Path;
use rtf_parser::{Lexer, Parser};
use zip::ZipArchive;
use std::io::Read;

/// Extracts plain text from a DOCX file using docx-rust with a manual fallback.
pub fn convert_docx_to_txt(path: &Path) -> Result<String> {
    // Ensure the file can be read
    let file_size = std::fs::metadata(path)
        .context("Failed to read file metadata")?
        .len();
    
    if file_size == 0 {
        return Err(anyhow::anyhow!("File is empty"));
    }
    
    // Try docx-rust first
    match try_docx_rust_parser(path) {
        Ok(text) => {
            if !text.trim().is_empty() {
                return Ok(text);
            } else {
                log::debug!("docx-rust extracted empty text, trying fallback for: {:?}", path);
            }
        }
        Err(e) => {
            log::debug!("docx-rust failed for {:?}, trying fallback: {}", path, e);
        }
    }
    
    // Fallback: manual extraction using zip
    log::debug!("Using fallback DOCX parser for: {:?}", path);
    extract_docx_text_manually(path)
}

/// Attempts to extract text using docx-rust
fn try_docx_rust_parser(path: &Path) -> Result<String> {
    let file = DocxFile::from_file(path)
        .context("Failed to read DOCX file")?;
    
    let docx = file.parse()
        .context("Failed to parse DOCX content")?;
    
    // Normalize the text: remove unnecessary line breaks
    let text = docx.document.body.text();
    let normalized = normalize_text(&text);
    
    Ok(normalized)
}

/// Normalizes text while preserving line breaks but removing extra spaces
fn normalize_text(text: &str) -> String {
    // Preserve line breaks, only normalize spaces within each line
    let lines: Vec<String> = text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    
    // Remove consecutive empty lines (maximum 1)
    let mut result_lines: Vec<String> = Vec::new();
    let mut last_was_empty = false;
    
    for line in lines {
        let is_empty = line.trim().is_empty();
        if is_empty && last_was_empty {
            continue;
        }
        result_lines.push(line);
        last_was_empty = is_empty;
    }
    
    result_lines.join("\n").trim().to_string()
}

/// Extracts text manually from the DOCX file using zip
fn extract_docx_text_manually(path: &Path) -> Result<String> {
    let file = fs::File::open(path)
        .context("Failed to open DOCX file")?;
    
    let mut zip = ZipArchive::new(file)
        .context("Failed to read DOCX as ZIP archive")?;
    
    // Try to read document.xml
    let mut document_file = zip.by_name("word/document.xml")
        .context("Failed to find word/document.xml in DOCX")?;
    
    let mut xml_content = String::new();
    document_file.read_to_string(&mut xml_content)
        .context("Failed to read document.xml content")?;
    
    // Remove all XML tags except the content of <w:t> tags
    let cleaned_text = extract_text_from_xml(&xml_content);
    
    if cleaned_text.trim().is_empty() {
        log::warn!("Manual extraction also returned empty text for: {:?}", path);
    }
    
    // Normalize the extracted text
    Ok(normalize_text(&cleaned_text))
}

/// Extracts clean text from XML, preserving line breaks (paragraphs)
fn extract_text_from_xml(xml_content: &str) -> String {
    let mut result = String::new();
    let mut in_text_tag = false;
    let mut chars = xml_content.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '<' {
            // Start of a tag
            let mut tag_content = String::new();
            
            // Collect the contents of the tag
            while let Some(&next_c) = chars.peek() {
                if next_c == '>' {
                    chars.next(); // Consume the '>'
                    break;
                }
                tag_content.push(chars.next().unwrap());
            }
            
            // Check if it is a <w:t> text tag
            if tag_content.starts_with("w:t") {
                in_text_tag = true;
            } else if tag_content.starts_with("/w:t") {
                in_text_tag = false;
            } else if tag_content.starts_with("/w:p") {
                // End of paragraph - add a line break
                result.push('\n');
            } else if tag_content.starts_with("w:br") {
                // Explicit line break
                result.push('\n');
            }
        } else if in_text_tag {
            // Inside a text tag, add the character
            result.push(c);
        }
    }
    
    // Clean extra spaces
    result = result.replace('\t', " ");
    
    // Remove multiple consecutive whitespace characters (within each line)
    let lines: Vec<String> = result.lines()
        .map(|line| {
            let mut l = line.to_string();
            while l.contains("  ") {
                l = l.replace("  ", " ");
            }
            l.trim().to_string()
        })
        .collect();
    
    // Remove consecutive empty lines
    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut last_was_empty = false;
    
    for line in lines {
        let is_empty = line.is_empty();
        if is_empty && last_was_empty {
            continue;
        }
        if !is_empty {
            cleaned_lines.push(line);
        }
        last_was_empty = is_empty;
    }
    
    cleaned_lines.join("\n").trim().to_string()
}

/// Extracts plain text from an RTF file using rtf-parser with a manual fallback.
pub fn convert_rtf_to_txt(path: &Path) -> Result<String> {
    // Read the RTF file contents
    let rtf_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            // Fallback for non-UTF8 files (e.g., Windows-1252)
            let bytes = fs::read(path).context("Failed to read RTF file bytes")?;
            String::from_utf8_lossy(&bytes).to_string()
        }
    };

    // Try rtf-parser first
    match try_rtf_parser(&rtf_content) {
        Ok(text) if !text.trim().is_empty() => {
            return Ok(normalize_text(&text));
        }
        Ok(_) => {
            log::debug!("rtf-parser returned empty text, trying fallback for: {:?}", path);
        }
        Err(e) => {
            log::debug!("rtf-parser failed for {:?}, trying fallback: {}", path, e);
        }
    }

    // Fallback: manual text extraction from RTF
    log::debug!("Using fallback RTF parser for: {:?}", path);
    let extracted = extract_rtf_text_manually(&rtf_content);
    
    if extracted.trim().is_empty() {
        log::warn!("RTF fallback also returned empty text for: {:?}", path);
    }
    
    Ok(normalize_text(&extracted))
}

/// Tenta extrair texto usando rtf-parser
fn try_rtf_parser(rtf_content: &str) -> Result<String> {
    let tokens = Lexer::scan(rtf_content)
        .context("Failed to tokenize RTF content")?;
    let mut parser = Parser::new(tokens);
    let document = parser.parse()
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to parse RTF document")?;
    
    Ok(document.get_text())
}

/// Extracts text manually from RTF by removing control codes
fn extract_rtf_text_manually(rtf_content: &str) -> String {
    let mut result = String::new();
    let mut chars = rtf_content.chars().peekable();
    let mut in_group: i32 = 0;
    let mut skip_group = false;
    let mut skip_depth: i32 = 0;
    
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                in_group += 1;
                if skip_group {
                    skip_depth += 1;
                    continue;
                }
                // Check if this is a group that should be ignored
                let mut peek_str = String::new();
                for _ in 0..30 {
                    if let Some(&pc) = chars.peek() {
                        if pc == '{' || pc == '}' {
                            break;
                        }
                        peek_str.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let peek_lower = peek_str.to_lowercase();
                if peek_lower.contains("fonttbl") || peek_lower.contains("colortbl") || 
                   peek_lower.contains("pict") || peek_lower.contains("stylesheet") ||
                   peek_lower.contains("\\info") || peek_lower.contains("header") ||
                   peek_lower.contains("footer") || peek_lower.contains("generator") ||
                   peek_lower.contains("\\*\\") {
                    skip_group = true;
                    skip_depth = 1;
                }
            }
            '}' => {
                if skip_group {
                    skip_depth -= 1;
                    if skip_depth <= 0 {
                        skip_group = false;
                        skip_depth = 0;
                    }
                }
                in_group = in_group.saturating_sub(1);
            }
            '\\' => {
                if skip_group {
                    continue;
                }
                // Process an RTF command
                let mut cmd = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphabetic() {
                        cmd.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Collect numeric parameters
                let mut param = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_numeric() || nc == '-' {
                        param.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Skip a single space after the command
                if let Some(&' ') = chars.peek() {
                    chars.next();
                }
                
                // Commands that generate text or line breaks
                match cmd.as_str() {
                    "par" | "line" => result.push('\n'),
                    "tab" => result.push(' '),
                    "" => {
                        // Could be an escaped character or hexadecimal character
                        if let Some(&nc) = chars.peek() {
                            if nc == '\\' || nc == '{' || nc == '}' {
                                result.push(chars.next().unwrap());
                            } else if nc == '\'' {
                                // Hexadecimal character: \'XX
                                chars.next(); // Consume the '
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if let Some(&hc) = chars.peek() {
                                        if hc.is_ascii_hexdigit() {
                                            hex.push(chars.next().unwrap());
                                        }
                                    }
                                }
                                if hex.len() == 2 {
                                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                        // Decode Windows-1252 to UTF-8
                                        let ch = decode_windows1252(byte);
                                        result.push(ch);
                                    }
                                }
                            }
                        }
                    }
                    // Ignore other commands
                    _ => {}
                }
            }
            '\'' => {
                // Standalone hexadecimal character: 'XX
                if skip_group {
                    continue;
                }
                let mut hex = String::new();
                for _ in 0..2 {
                    if let Some(&hc) = chars.peek() {
                        if hc.is_ascii_hexdigit() {
                            hex.push(chars.next().unwrap());
                        }
                    }
                }
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        let ch = decode_windows1252(byte);
                        result.push(ch);
                    }
                }
            }
            '\r' | '\n' => {
                // Ignore literal line breaks in the RTF
            }
            _ => {
                if !skip_group && in_group > 0 {
                    result.push(c);
                }
            }
        }
    }
    
    // Remove font metadata that may have leaked through
    let result = remove_font_metadata(&result);
    
    // Remove multiple consecutive line breaks
    let mut result = result;
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    
    result.trim().to_string()
}

/// Decodes a Windows-1252 byte into a UTF-8 character
fn decode_windows1252(byte: u8) -> char {
    // Windows-1252 to Unicode mapping (special characters)
    match byte {
        0x80 => '€',
        0x82 => '‚',
        0x83 => 'ƒ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => 'ˆ',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Œ',
        0x8E => 'Ž',
        0x91 => '\'',
        0x92 => '\'',
        0x93 => '"',
        0x94 => '"',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x98 => '˜',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'œ',
        0x9E => 'ž',
        0x9F => 'Ÿ',
        // For other bytes, use directly as Latin-1
        b => b as char,
    }
}

/// Removes font metadata and RTF commands that may have leaked into the text
fn remove_font_metadata(text: &str) -> String {
    let mut result = text.to_string();
    
    // Remove leaked RTF formatting commands (s20, fs20, b, b0, etc.)
    // Process line by line for better control
    let lines: Vec<String> = result.lines()
        .map(|line| clean_rtf_line(line))
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !is_rtf_garbage_line(line))
        .collect();
    
    result = lines.join("\n");
    
    // Collapse repeated spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    result
}

/// Cleans a line of leaked RTF commands
fn clean_rtf_line(line: &str) -> String {
    let mut result = line.to_string();
    
    // Remove common leaked RTF commands
    let rtf_patterns = [
        // Font/size commands (with variable numbers)
        "fs22", "fs20", "fs18", "fs16", "fs14", "fs12", "fs10",
        "s22", "s20", "s18", "s16", "s14", "s12", "s10",
        // Bold/italic commands
        "b0", "b ", "i0", "i ",
        // Language commands
        "eflang1046", "deflang1046", "eflang1033", "deflang1033",
        "lang1046", "lang1033",
        // Configuration commands
        "hfdbch0mdefJc1", "hfdbch0", "mdef", "Jc1",
        // Font metadata
        "Arial-BoldMT", "ArialMT", "Arial-ItalicMT",
        "Helvetica", "HelveticaNeue", "Times-Roman", "Calibri",
        "Msftedit",
    ];
    
    for pattern in rtf_patterns.iter() {
        result = result.replace(pattern, "");
    }
    
    // Convert ~ (RTF non-breaking space) into a normal space
    result = result.replace("~", " ");
    
    // Remove trailing standalone 'b' (bold command)
    if result.ends_with("b") && !result.ends_with("mb") {
        result = result.trim_end_matches('b').to_string();
    }
    
    // Remove stray sequences of semicolons
    while result.contains(";;;") {
        result = result.replace(";;;", "");
    }
    while result.contains(";;") {
        result = result.replace(";;", "");
    }
    result = result.replace(";*", "");
    
    // Remove repeated spaces and tidy up
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    result.trim().to_string()
}

/// Checks whether a line is RTF garbage that should be removed entirely
fn is_rtf_garbage_line(line: &str) -> bool {
    let line = line.trim();
    
    // Empty line or only control characters
    if line.is_empty() {
        return true;
    }
    
    // Line that contains only RTF commands (no actual text)
    let cleaned = line
        .replace("fs", "")
        .replace("s", "")
        .replace("b", "")
        .replace("0", "")
        .replace("1", "")
        .replace("2", "")
        .replace("~", "")
        .replace(" ", "");
    
    if cleaned.chars().all(|c| c.is_numeric()) {
        return true;
    }
    
    // Lines that start with RTF configuration commands
    if line.starts_with("hfdbch") || line.starts_with("mdef") {
        return true;
    }
    
    // Lines that are only font metadata
    if line.contains(';') && 
       (line.contains("Arial") || line.contains("Calibri") || 
        line.contains("Helvetica") || line.contains("Msftedit") ||
        line.contains("Times")) {
        return true;
    }
    
    false
}
