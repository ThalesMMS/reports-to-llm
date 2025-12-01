# Reports to LLM

A Rust-based tool for converting and normalizing medical reports from DOCX/RTF formats into clean, structured plain text optimized for Large Language Models (LLM) and Retrieval-Augmented Generation (RAG) systems.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [How It Works](#how-it-works)
- [Output Format](#output-format)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Dependencies](#dependencies)

---

## Overview

Medical reports often come in various document formats (DOCX, RTF) with inconsistent formatting, embedded metadata, and encoding issues. This tool addresses these challenges by:

1. **Extracting text** from DOCX and RTF files with multiple fallback strategies
2. **Cleaning artifacts** like font metadata, formatting commands, and XML tags
3. **Normalizing encoding** (Windows-1252 to UTF-8) to preserve accented characters
4. **Structuring content** into consistent, readable sections
5. **Aggregating results** into manageable output files without splitting individual reports

The resulting clean text is ideal for:
- Training or fine-tuning LLMs on medical data
- Building RAG systems for medical knowledge retrieval
- Creating searchable medical document databases
- Text analysis and NLP applications

---

## Features

### Document Conversion
- **DOCX Support**: Primary parser with automatic fallback to manual ZIP extraction
- **RTF Support**: Full parser with manual fallback for problematic files
- **Encoding Handling**: Automatic Windows-1252 to UTF-8 conversion for Portuguese/Latin characters

### Text Cleaning
- **RTF Command Removal**: Strips formatting commands (`\s20`, `\fs20`, `\b`, etc.)
- **XML Tag Removal**: Cleans DOCX XML artifacts (`<w:t>`, `<w:p>`, etc.)
- **Metadata Filtering**: Removes font names, editor info, and other embedded metadata
- **Accent Preservation**: Correctly decodes characters like ã, é, ç, ó, etc.

### Content Structuring
- **Section Detection**: Identifies standard medical report sections
- **Consistent Formatting**: Normalizes section headers and spacing
- **Line Break Preservation**: Maintains original paragraph structure
- **Report Separation**: Adds clear spacing between individual reports

### Output Management
- **Smart File Splitting**: Divides output into multiple files (max 50,000 lines each)
- **Intact Reports**: Never splits a single report across files
- **Error Logging**: All warnings and errors saved to `log.txt`

---

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later recommended)
- Cargo (included with Rust)

### Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ThalesMMS/reports-to-llm.git
   cd reports-to-llm
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Create input directory** (if it doesn't exist):
   ```bash
   mkdir -p docs
   ```

---

## Usage

### Basic Usage

1. **Place your files** in the `./docs/` directory:
   ```
   docs/
   ├── patient1.docx
   ├── patient2.rtf
   ├── department/
   │   ├── report1.docx
   │   └── report2.docx
   └── ...
   ```

2. **Run the converter**:
   ```bash
   cargo run --release
   ```

3. **Find the results** in `./output/`:
   ```
   output/
   ├── result_1.txt
   ├── result_2.txt
   └── ...
   ```

4. **Check for errors** in `log.txt`:
   ```bash
   cat log.txt
   ```

### Command Output

During execution, you'll see progress information:
```
[INFO] Converting: "department/patient1.docx"
[INFO] Converting: "department/patient2.rtf"
[INFO] Conversion phase completed. 150 files converted.
[INFO] Aggregating: "department/patient1.txt"
[INFO] Aggregating: "department/patient2.txt"
[INFO] Concatenation finished. Total files created: 3
[INFO] Processing finished successfully. Results are in the './output' directory.
```

---

## How It Works

### Processing Pipeline

```
┌─────────────────┐
│   Input Files   │
│  (DOCX / RTF)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Conversion    │
│  DOCX → Text    │
│  RTF  → Text    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Cleaning     │
│ - Remove RTF    │
│ - Remove XML    │
│ - Fix encoding  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Formatting    │
│ - Section IDs   │
│ - Spacing       │
│ - Structure     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Aggregation    │
│ - Concatenate   │
│ - Split files   │
│ - Add spacing   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Output Files   │
│ (result_*.txt)  │
└─────────────────┘
```

### DOCX Conversion Strategy

1. **Primary**: Use `docx-rust` library to parse the document
2. **Fallback**: If that fails, treat DOCX as a ZIP archive and extract `word/document.xml`
3. **XML Parsing**: Extract text from `<w:t>` tags, convert `</w:p>` to line breaks

### RTF Conversion Strategy

1. **Primary**: Use `rtf-parser` library to parse the document
2. **Fallback**: Manual character-by-character parsing that:
   - Skips control groups (fonts, colors, stylesheets)
   - Converts `\par` and `\line` to newlines
   - Decodes hex characters (`\'e3` → `ã`)
   - Removes leaked formatting commands

### Encoding Conversion

RTF files often use Windows-1252 encoding for special characters. The tool maps these to UTF-8:

| RTF Code | Windows-1252 | UTF-8 Character |
|----------|--------------|-----------------|
| `\'e3`   | 0xE3         | ã               |
| `\'e9`   | 0xE9         | é               |
| `\'f3`   | 0xF3         | ó               |
| `\'e7`   | 0xE7         | ç               |
| `\'ed`   | 0xED         | í               |
| `\'e1`   | 0xE1         | á               |
| `\'f5`   | 0xF5         | õ               |
| `\'e2`   | 0xE2         | â               |
| `\'ea`   | 0xEA         | ê               |
| `\'fa`   | 0xFA         | ú               |

---

## Output Format

### Structured Report Example

Each medical report is formatted with clear sections and consistent spacing:

```
COMPUTED TOMOGRAPHY OF ABDOMEN AND PELVIS

Exam technique:
Volumetric acquisitions with subsequent multiplanar reconstructions.
Images obtained without intravenous contrast.

Observed findings:
Liver with normal dimensions, regular contours, and homogeneous density.
Gallbladder with normal characteristics.
Spleen with normal dimensions, regular contours, and typical density.
Pancreas with normal dimensions and anatomy, without dilation of the main duct.
Adrenal glands with normal appearance.
Kidneys in normal position, with typical dimensions, structure, and contours.
No dilation of the pyelocalyceal structures.
Perirenal spaces are clear.
Bladder with normal shape, contours, and capacity.
Pelvic fat planes preserved.
No abdominal, pelvic, or inguinal lymphadenopathy.
Vascular structures with normal appearance.
Osseous structures intact.

IMPRESSION:
No pathological findings detectable by this method.
The diagnostic impression is probabilistic and should be interpreted together with
clinical and laboratory data, as well as previous and/or subsequent imaging studies.


NEXT REPORT TITLE...
```

### Key Formatting Rules

1. **Report Title**: First line, usually in CAPS
2. **Empty Line**: After title
3. **Section Headers**: `Exam technique:`, `Observed findings:`, `IMPRESSION:`
4. **Section Content**: Follows header on next line(s)
5. **Report Separation**: Two empty lines between reports

---

## Project Structure

```
reports-to-llm/
├── Cargo.toml              # Project dependencies and metadata
├── README.md               # This documentation
├── log.txt                 # Runtime errors and warnings (generated)
│
├── docs/                   # INPUT: Place your DOCX/RTF files here
│   └── (your files)
│
├── output/                 # OUTPUT: Generated text files
│   ├── result_1.txt
│   ├── result_2.txt
│   └── ...
│
├── src/
│   ├── main.rs             # Application entry point, setup, and orchestration
│   ├── processor.rs        # File discovery, formatting, and aggregation
│   └── converters.rs       # DOCX and RTF conversion logic
│
└── tests/
    └── output_quality.rs   # Quality assurance tests for outputs
```

### Source Files

| File | Purpose |
|------|---------|
| `main.rs` | Initializes logging, creates directories, orchestrates the pipeline |
| `processor.rs` | Walks directories, formats medical reports, concatenates outputs |
| `converters.rs` | Converts DOCX/RTF to plain text with fallback strategies |

---

## Testing

### Running Tests

The project includes comprehensive quality tests to ensure output consistency:

```bash
# Run all tests
cargo test

# Run quality tests with detailed output
cargo test --test output_quality -- --nocapture
```

### Test Descriptions

| Test | What It Checks |
|------|----------------|
| `test_no_rtf_commands_in_output` | No RTF formatting commands leaked (`s20`, `fs20`, `hfdbch`, etc.) |
| `test_no_encoding_errors_in_output` | No undecoded characters (`'e3`, `'f3`, `'e9`, etc.) |
| `test_no_font_metadata_in_output` | No font metadata leaked (`Arial`, `Calibri`, `Msftedit`, etc.) |
| `test_no_xml_tags_in_output` | No DOCX XML tags leaked (`<w:t>`, `<w:p>`, etc.) |
| `test_report_structure` | Reports contain expected section headers |
| `test_report_spacing` | Proper spacing between sections and reports |
| `test_output_files_exist` | Output files were generated |
| `test_output_files_not_empty` | Output files contain content |
| `test_full_output_quality` | Comprehensive check with detailed report |

### Example Test Output

```
========================================
  OUTPUT QUALITY VERIFICATION
========================================

📁 7 file(s) found

✅ ./output/result_1.txt
✅ ./output/result_2.txt
✅ ./output/result_3.txt
✅ ./output/result_4.txt
✅ ./output/result_5.txt
✅ ./output/result_6.txt
✅ ./output/result_7.txt

========================================
  ✅ ALL TESTS PASSED!
========================================
```

---

## Troubleshooting

### Common Issues

#### Empty Output Files
- **Cause**: Input files may be corrupted or in unsupported format
- **Solution**: Check `log.txt` for specific file errors

#### Missing Accents (ã → a)
- **Cause**: Encoding not properly detected
- **Solution**: This should be handled automatically; report issue if persists

#### RTF Commands in Output (`s20`, `fs20`)
- **Cause**: Unusual RTF structure not caught by filters
- **Solution**: Run tests to identify specific files; report issue

#### "Failed to parse DOCX/RTF" Errors
- **Cause**: File may be corrupted or use non-standard format
- **Solution**: The tool attempts fallback parsing; check if output was still generated

#### Files Starting with `~$`
- **Cause**: These are temporary Office files (lock files)
- **Solution**: These are automatically skipped; close Office applications to remove them

### Checking Logs

```bash
# View all errors
cat log.txt

# Count errors by type
grep -c "ERROR" log.txt
grep -c "WARN" log.txt

# Find specific file errors
grep "patient_name" log.txt
```

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `anyhow` | 1.0 | Ergonomic error handling |
| `walkdir` | 2.3 | Recursive directory traversal |
| `docx-rust` | 0.1.10 | DOCX file parsing |
| `rtf-parser` | 0.1 | RTF file parsing |
| `zip` | 0.6 | ZIP archive handling (DOCX fallback) |
| `log` | 0.4 | Logging facade |
| `simplelog` | 0.12 | Combined terminal + file logging |

---
