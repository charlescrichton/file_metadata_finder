# file_metadata_finder

A Rust-based command-line tool that recursively scans directories to extract file metadata and produces a JSON output. The tool is designed to work on Mac, Windows, and Linux, with full cross-platform path handling and support for cross-compilation.

## Features

- **Cross-platform compatibility**: Native Windows, macOS, and Linux support with proper path handling
- **Cross-compilation support**: Build Windows .exe files from macOS/Linux
- **Recursive directory scanning** with progress tracking
- **Smart file filtering**: Only processes supported file types (CSV, Excel, PDF, DOCX, EML)
- **File integrity checking**: CRC32 hash calculation for files ≤ 128KB (enabled by default, larger files report size)
- **Dataset similarity detection**: Column similarity hash for CSV/Excel files to identify structurally similar datasets
- **Multi-format support:**
  - CSV files: Extracts column names, row count, and column similarity hash
  - Excel files (.xlsx, .xls, .xlsm, .xlsb): Extracts per-sheet column names (with smart header detection in first 5 rows), row counts, and column similarity hash
  - PDF files: Detects and logs presence
  - DOCX files: Detects and logs presence  
  - EML files: Detects and logs presence
- **NHS Number redaction**: Automatically redacts 10-digit NHS numbers and numbers in `nnn nnn nnnn` format in:
  - File paths
  - File names
  - Column names
  - Sheet names
- **JSON output**: Flat array of directory objects with file details (excludes directories with no matching files)
- **Simplified timestamps**: Date and time in YYYY-MM-DDTHH:MM format

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

### Cross-compilation for Windows (from macOS/Linux)

To build a Windows .exe file that can be run in PowerShell 7 or Command Prompt:

```bash
# Use the provided build script
./build-windows.sh
```

Or manually:

```bash
# Add the Windows target
rustup target add x86_64-pc-windows-gnu

# Install mingw-w64 (if not already installed)
# On macOS: brew install mingw-w64
# On Ubuntu/Debian: sudo apt install gcc-mingw-w64-x86-64
# On Fedora: sudo dnf install mingw64-gcc

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# The Windows executable will be at target/x86_64-pc-windows-gnu/release/file_metadata_finder.exe
```

### Cross-compilation for Linux MUSL

To build a static binary for Linux (x86_64-unknown-linux-musl):

```bash
# Use the provided build script
./build-musl.sh
```

Or manually:

```bash
# Add the target
rustup target add x86_64-unknown-linux-musl

# Build for the target
cargo build --release --target x86_64-unknown-linux-musl

# The binary will be at target/x86_64-unknown-linux-musl/release/file_metadata_finder
```

## Usage

### On Windows (PowerShell 7 or Command Prompt)

```powershell
# Basic scan with default settings (CRC32 hash enabled for files ≤ 128KB)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json"

# Disable CRC32 hash calculation (only report file sizes)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --disable-hash

# Scan a different drive
.\file_metadata_finder.exe --directory "D:\MyData" --output "metadata.json"
```

### On macOS/Linux

```bash
# Basic scan with default settings (CRC32 hash enabled for files ≤ 128KB)
./file_metadata_finder --directory /path/to/data --output results.json

# Disable CRC32 hash calculation (only report file sizes)
./file_metadata_finder --directory /path/to/data --output results.json --disable-hash

# Scan with default output file
./file_metadata_finder --directory /path/to/data
```

### Arguments

- `-d, --directory <PATH>`: Directory to scan (required)
- `-o, --output <OUTPUT_FILE>`: Output JSON file path (default: "output.json")
- `--disable-hash`: Disable CRC32 hash calculation (default: enabled for files ≤ 128KB)

### Arguments

- `-d, --directory <PATH>`: Directory to scan (required)
- `-o, --output <OUTPUT_FILE>`: Output JSON file path (default: "output.json")
- `--enable-hash`: Enable CRC32 hash calculation for files ≤ 128KB (larger files report size only)

## Path Handling

The tool correctly handles paths across all platforms:
- **Windows**: `C:\Users\Name\Documents`, `D:\Data\Files`
- **macOS/Linux**: `/home/user/documents`, `/Users/name/Documents`

All path operations use Rust's cross-platform `std::path` module for reliable path handling.

## Output Format

The tool produces a JSON file with the following structure:

```json
{
  "scan_directory": "/absolute/path/to/scanned/directory",
  "directories": [
    {
      "path": "/path/to/directory",
      "files": [
        {
          "name": "example.csv",
          "created": "2025-10-03T13:39",
          "file_type": "csv",
          "crc32_hash": "191c8b02",
          "csv_metadata": {
            "columns": ["Column1", "Column2", "Column3"],
            "row_count": 100,
            "column_similarity_hash": 1698031807
          }
        },
        {
          "name": "data_[REDACTED].xlsx",
          "created": "2025-10-03T13:41",
          "file_type": "excel",
          "file_size": 2048000,
          "excel_metadata": {
            "sheets": [
              {
                "sheet_name": "Sheet1",
                "columns": ["ID", "Name", "Value"],
                "row_count": 50,
                "column_similarity_hash": 288347173
              }
            ]
          }
        },
        {
          "name": "document.pdf",
          "created": "2025-10-03T13:39",
          "file_type": "pdf",
          "file_size": 524288
        }
      ]
    }
  ]
}
```

### Field Descriptions

- **`scan_directory`**: Absolute path of the directory that was scanned
- **`directories`**: Array of directories containing matching files

- **`created`**: File creation timestamp in simplified format (YYYY-MM-DDTHH:MM)
- **`crc32_hash`**: Present for files ≤ 128KB (default behavior). 8-character hexadecimal CRC32 hash
- **`file_size`**: Present for files > 128KB or when `--disable-hash` is used. Size in bytes
- **`column_similarity_hash`**: Present for CSV and Excel files. CRC32 hash of processed column names (lowercase, alphanumeric only, sorted) to identify structurally similar datasets
- **File type metadata**: Additional fields (like `csv_metadata`, `excel_metadata`) are included based on file type

### File Filtering

Only supported file types are processed and included in the output:
- **CSV files**: `.csv`
- **Excel files**: `.xlsx`, `.xls`, `.xlsm`, `.xlsb` 
- **Document files**: `.pdf`, `.docx`, `.eml`

Directories containing only unsupported file types are excluded from the output.

### Column Similarity Hash

For CSV and Excel files, a `column_similarity_hash` is calculated to identify datasets with similar structure:

1. Column names are converted to lowercase
2. Non-alphanumeric characters are removed  
3. Empty column names are filtered out
4. Remaining column names are sorted alphabetically
5. Names are concatenated with commas: `"col1,col2,col3"`
6. CRC32 hash is calculated on the concatenated string

Example: Files with columns `["Name", "Age", "City"]` and `["city", "name", "age"]` will have the same similarity hash.

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
- `crc32fast`: Fast CRC32 hash calculation

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
