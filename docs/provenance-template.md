# Private provenance template

Use this template for a **local, non-public** record of the inputs and execution context behind a conversion run.

This repository does not bundle patient data, and this template is not meant to be committed with identifiable filenames or PHI. If filenames or directory names are sensitive, replace them with study-specific aliases.

## Run identity

- Project or study name:
- Operator:
- Date run started (ISO 8601 with timezone, e.g. `2026-04-13T15:30:00+00:00`):
- Date run finished (ISO 8601 with timezone, e.g. `2026-04-13T16:45:00+00:00`):
- Repository commit SHA:
- Binary or command used:
  - Example: `cargo run --release`
- Host environment:
  - Example: `rustc --version`, OS version, locale

## Input snapshot

- Source institution or archive:
- Access approval / governance reference:
- Inclusion criteria:
- Exclusion criteria:
- Date range covered by the documents:
- Modalities or report types included:
- File formats included:
  - Example: DOCX, RTF
- Total files discovered:
- Total files converted successfully:
- Total files skipped or failed:
- Location of the local input snapshot:
  - Example: encrypted folder path or internal storage reference

## Integrity notes

- Checksum method:
  - Example: `shasum -a 256`
- Manifest file location:
  - Example: `manifests/input-sha256.txt`
- Output checksum file location:
  - Example: `manifests/output-sha256.txt`

Example commands:

```bash
find docs -type f \( -name '*.docx' -o -name '*.rtf' \) -print0 | \
  sort -z | xargs -0 shasum -a 256 > manifests/input-sha256.txt

find output -type f -name 'result_*.txt' -print0 | \
  sort -z | xargs -0 shasum -a 256 > manifests/output-sha256.txt
```

## Manual review notes

- Files manually spot-checked against originals:
- Sections verified:
  - Example: title, technique, findings, impression
- Character / accent preservation issues observed:
- Formatting artifacts observed:
- Missing or partial extraction cases observed:
- Follow-up actions needed:

## Limitations acknowledged for this run

- Whether PHI may still be present in outputs:
- Whether OCR-only or scanned-image reports were excluded:
- Whether non-standard templates were present:
- Whether this output was used only for research / educational workflows:
- Confirmation that results were not used for clinical decision-making:

## Downstream linkage

- Where the resulting text was used next:
  - Example: local search index, RAG prototype, annotation prep
- Separate de-identification step used, if any:
- Separate clinical or scientific validation performed, if any:
