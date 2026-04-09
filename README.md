# Reports to LLM

A Rust-based tool for converting and normalizing local medical reports from DOCX/RTF into clean, structured plain text. It is useful as a preprocessing step for downstream LLM/RAG experiments, search, or corpus curation, but it is not a de-identification tool, a clinical system, or a model evaluation benchmark.

## Table of Contents

- [Overview](#overview)
- [Intended Use, Data Provenance, and Safety](#intended-use-data-provenance-and-safety)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Reproducibility Quickstart](#reproducibility-quickstart)
- [How It Works](#how-it-works)
- [Expected Input / Output Example](#expected-input--output-example)
- [Output Format](#output-format)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Limitations and Evaluation Caveats](#limitations-and-evaluation-caveats)
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

The resulting clean text can be useful as a preprocessing step for:
- Building research corpora from locally held DOCX/RTF reports
- Feeding downstream LLM or RAG experiments after separate governance review
- Creating searchable internal text collections
- Text analysis and NLP prototyping

---

## Intended Use, Data Provenance, and Safety

- **Scope**: this repository converts DOCX/RTF report files into normalized plain text. It does **not** perform diagnosis, recommendation generation, or clinical quality assurance.
- **Data provenance**: the repository does not ship a public patient dataset. All outputs depend on the local input files that the operator places in `./docs/`, so provenance, completeness, and formatting quality vary by source institution and reporting template.
- **Privacy**: if the source reports contain names, identifiers, or other protected health information, the converted text may still contain that information. This tool does **not** de-identify or anonymize reports.
- **Research / educational use only**: use the generated text as an intermediate artifact for research, tooling, or documentation workflows. Do **not** use the output on its own for clinical decision-making, diagnosis, or treatment.
- **Traceability**: for any downstream study, keep the original source documents, the exact commit used, and a record of the commands run so that text extraction decisions can be audited later.

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

## Reproducibility Quickstart

Because medical reports may contain sensitive information, the repository does not include a bundled sample dataset. The commands below create **synthetic** local fixtures so you can verify the pipeline without real patient data.

```bash
# Start from a clean working tree state
find output -maxdepth 1 -type f -name 'result_*.txt' -delete 2>/dev/null || true
rm -f log.txt
mkdir -p docs

# Minimal synthetic RTF example
cat > docs/synthetic-report.rtf <<'EOF'
{\rtf1\ansi\deff0{\fonttbl{\f0 Arial;}}
\b TOMOGRAFIA DE TÓRAX\b0\par
TÉCNICA DO EXAME: Cortes axiais sem contraste.\par
ASPECTOS OBSERVADOS: Pequeno nódulo sólido no lobo superior direito.\par
IMPRESSÃO DIAGNÓSTICA: Achado inespecífico.\par
}
EOF

# Minimal synthetic DOCX example written as a ZIP container
python3 - <<'PY'
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

docx_path = Path("docs/synthetic-report.docx")
document_xml = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:r><w:t>ULTRASSONOGRAFIA ABDOMINAL</w:t></w:r></w:p>
<w:p><w:r><w:t>TÉCNICA DO EXAME: Estudo convencional.</w:t></w:r></w:p>
<w:p><w:r><w:t>ASPECTOS OBSERVADOS: Fígado sem alterações focais.</w:t></w:r></w:p>
<w:p><w:r><w:t>IMPRESSÃO DIAGNÓSTICA: Sem achados agudos.</w:t></w:r></w:p>
</w:body></w:document>
"""
content_types = """<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>
"""
rels = """<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>
"""

with ZipFile(docx_path, "w", ZIP_DEFLATED) as zf:
    zf.writestr("[Content_Types].xml", content_types)
    zf.writestr("_rels/.rels", rels)
    zf.writestr("word/document.xml", document_xml)
PY

# Run the conversion
cargo run --release

# Inspect the first generated file
sed -n '1,80p' output/result_1.txt

# Run regression-style output checks
cargo test --test output_quality -- --nocapture
```

For a reproducible audit trail, record the commit SHA, the exact command sequence above, and a checksum of the local input files used in your own experiments.

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

## Expected Input / Output Example

The example below is intentionally **synthetic**. It demonstrates the kind of cleanup the tool performs without using real patient data.

### Example input snippet (RTF)

```rtf
{\rtf1\ansi\deff0{\fonttbl{\f0 Arial;}}
\b TOMOGRAFIA DE TÓRAX\b0\par
TÉCNICA DO EXAME: Cortes axiais sem contraste.\par
ASPECTOS OBSERVADOS: Pequeno nódulo sólido no lobo superior direito.\par
IMPRESSÃO DIAGNÓSTICA: Achado inespecífico.\par
}
```

### Expected normalized output

```text
TOMOGRAFIA DE TÓRAX

Técnica do exame:
Cortes axiais sem contraste.

Aspectos observados:
Pequeno nódulo sólido no lobo superior direito.

IMPRESSÃO:
Achado inespecífico.
```

This example shows the expected behavior: removal of RTF formatting commands, preservation of accented characters, section-header normalization, and stable plain-text output suitable for downstream inspection.

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

These tests are best understood as **regression checks on formatting quality**. They help detect leaked RTF/XML artifacts, spacing problems, and missing outputs, but they do not establish extraction recall, semantic accuracy, or clinical validity.

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

## Limitations and Evaluation Caveats

- **No de-identification**: the pipeline does not remove names, IDs, accession numbers, dates, or other PHI/PII from source reports.
- **Format-specific scope**: the current implementation targets text extraction from DOCX and RTF. It does not perform OCR on scanned PDFs or images.
- **Template dependence**: section normalization is tuned to common report headings present in the codebase (for example, radiology-style headings in Portuguese). Other specialties, institutions, or languages may produce less structured output.
- **Fallback parsers may be partial**: malformed or unusual DOCX/RTF files can still yield incomplete extraction or be skipped and logged in `log.txt`.
- **No benchmark claims**: the repository does not include a public evaluation corpus or benchmark numbers, so output quality should be verified on representative local data before downstream research use.
- **Manual review still required**: before using converted text in a study or internal pipeline, spot-check outputs against the original documents to confirm that key sections, accents, and report boundaries were preserved.

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
