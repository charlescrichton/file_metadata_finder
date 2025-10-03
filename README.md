# file_metadata_finder

A Rust-based command-line tool that recursively scans directories to extract file metadata and produces a JSON output. The tool is designed to work on Mac, Windows, and Linux, with support for cross-compilation to x86_64-unknown-linux-musl.

## Features

- **Recursive directory scanning** with progress tracking
- **Multi-format support:**
  - CSV files: Extracts column names and row count
  - Excel files (.xlsx, .xls, .xlsm, .xlsb): Extracts per-sheet column names (with smart header detection in first 5 rows) and row counts
  - PDF files: Detects and logs presence
  - DOCX files: Detects and logs presence
  - EML files: Detects and logs presence
- **NHS Number redaction**: Automatically redacts 10-digit NHS numbers and numbers in `nnn nnn nnnn` format in:
  - File paths
  - File names
  - Column names
  - Sheet names
- **JSON output**: Flat array of directory objects with file details
- **ISO 8601 datetime format** for file creation timestamps

## Installation

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs/))

### Building from source

```bash
# Clone the repository
git clone https://github.com/charlescrichton/file_metadata_finder.git
cd file_metadata_finder

# Build in release mode
cargo build --release

# The binary will be available at target/release/file_metadata_finder
```

### Cross-compilation for Linux MUSL

To build a static binary for Linux (x86_64-unknown-linux-musl):

```bash
# Add the target
rustup target add x86_64-unknown-linux-musl

# Build for the target
cargo build --release --target x86_64-unknown-linux-musl

# The binary will be at target/x86_64-unknown-linux-musl/release/file_metadata_finder
```

## Usage

```bash
file_metadata_finder --directory <PATH> [--output <OUTPUT_FILE>]
```

### Arguments

- `-d, --directory <PATH>`: Directory to scan (required)
- `-o, --output <OUTPUT_FILE>`: Output JSON file path (default: "output.json")

### Example

```bash
# Scan a directory and output to default file
file_metadata_finder --directory /path/to/data

# Scan with custom output file
file_metadata_finder --directory /path/to/data --output results.json
```

## Output Format

The tool produces a JSON file with the following structure:

```json
[
  {
    "path": "/path/to/directory",
    "files": [
      {
        "name": "example.csv",
        "created": "2025-10-03T13:39:14.765303734+00:00",
        "file_type": "csv",
        "csv_metadata": {
          "columns": ["Column1", "Column2", "Column3"],
          "row_count": 100
        }
      },
      {
        "name": "data_[REDACTED].xlsx",
        "created": "2025-10-03T13:41:30.944665634+00:00",
        "file_type": "excel",
        "excel_metadata": {
          "sheets": [
            {
              "sheet_name": "Sheet1",
              "columns": ["ID", "Name", "Value"],
              "row_count": 50
            }
          ]
        }
      },
      {
        "name": "document.pdf",
        "created": "2025-10-03T13:39:33.454624520+00:00",
        "file_type": "pdf"
      }
    ]
  }
]
```

### NHS Number Redaction

NHS numbers are automatically replaced with `[REDACTED]` in:
- File names (e.g., `patient_1234567890.csv` → `patient_[REDACTED].csv`)
- Directory paths
- Column names in CSV files
- Column names in Excel sheets
- Sheet names in Excel files

The tool detects:
- 10 consecutive digits (e.g., `1234567890`)
- Spaced format (e.g., `123 456 7890`)

## Dependencies

- `serde` & `serde_json`: JSON serialization
- `csv`: CSV file parsing
- `calamine`: Excel file parsing
- `chrono`: DateTime handling
- `regex`: NHS number pattern matching
- `walkdir`: Directory traversal
- `clap`: Command-line argument parsing
- `indicatif`: Progress bar display
- `anyhow`: Error handling

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
