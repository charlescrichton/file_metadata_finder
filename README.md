# file_metadata_finder

A Rust-based command-line tool that recursively scans directories to extract file metadata and produces a JSON output. The tool is designed to work on Mac, Windows, and Linux, with full cross-platform path handling and support for cross-compilation.

## Features

- **Cross-platform compatibility**: Native Windows, macOS, and Linux support with proper path handling
- **Cross-compilation support**: Build Windows .exe files from macOS/Linux
- **Recursive directory scanning** with progress tracking
- **Smart file filtering**: Only processes supported file types (CSV, Excel, PDF, DOCX, EML)
- **File integrity checking**: CRC32 hash calculation for files ≤ 128KB (enabled by default, larger files report size)
- **Dataset similarity detection**: Column similarity hash for CSV/Excel files to identify structurally similar datasets
- **Column similarity table**: Maps similarity hashes to files/sheets that share the same column structure
- **CRC32 similarity table**: Maps CRC32 hashes to files with identical content for duplicate detection
- **Fuzzy similarity grouping**: Groups datasets with similar but not identical column names using fuzzy string matching
- **Row count limiting**: Configurable maximum rows to process (default: 524,288) with `stopped_row_count_at` indicator
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

## 🔒 Safety Assurance

**This tool is completely read-only and will never modify, delete, or alter any scanned files.**

### Technical Guarantees:

- **Read-Only Operations**: All file access uses read-only operations (`File::open`, `fs::metadata`)
- **No Write Operations**: The only file write operation is creating the output JSON file (specified by `--output`)
- **Immutable Processing**: File contents are only read into memory for analysis, never modified
- **Safe Libraries**: Uses trusted Rust libraries:
  - `calamine` for Excel reading (read-only)
  - `csv` crate with `ReaderBuilder` (read-only)
  - `std::fs::metadata` for file properties (read-only)
  - `walkdir` for directory traversal (read-only)

### What the tool does:
✅ Reads file metadata (creation time, size)  
✅ Reads file contents for hash calculation  
✅ Parses CSV/Excel headers and row counts  
✅ Creates a separate JSON output file  

### What the tool never does:
❌ Modifies any scanned files  
❌ Deletes any files  
❌ Creates files in scanned directories  
❌ Changes file permissions or attributes  
❌ Moves or renames files  

The tool is designed for safe analysis of sensitive data environments where file integrity is paramount.

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
# Basic scan with default settings (CRC32 hash enabled, max 255 columns, max 524,288 rows)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json"

# Disable CRC32 hash calculation (only report file sizes)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --disable-hash

# Limit columns to 10 for large datasets
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --max-columns 10

# Limit rows to 1000 for very large files
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --max-rows 1000

# Enable fuzzy similarity grouping with threshold 0.8 (default)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --fuzzy-threshold 0.8

# Strict fuzzy similarity grouping (only very similar column names)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --fuzzy-threshold 0.9

# Disable fuzzy similarity grouping
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --fuzzy-threshold 0

# Show all columns (unlimited)
.\file_metadata_finder.exe --directory "C:\Users\YourName\Documents" --output "scan_results.json" --max-columns 0

# Scan a different drive
.\file_metadata_finder.exe --directory "D:\MyData" --output "metadata.json"
```

### On macOS/Linux

```bash
# Basic scan with default settings (CRC32 hash enabled, max 255 columns, max 524,288 rows)
./file_metadata_finder --directory /path/to/data --output results.json

# Disable CRC32 hash calculation (only report file sizes)
./file_metadata_finder --directory /path/to/data --output results.json --disable-hash

# Limit columns to 10 for large datasets
./file_metadata_finder --directory /path/to/data --output results.json --max-columns 10

# Limit rows to 1000 for very large files
./file_metadata_finder --directory /path/to/data --output results.json --max-rows 1000

# Enable fuzzy similarity grouping with threshold 0.8 (default)
./file_metadata_finder --directory /path/to/data --output results.json --fuzzy-threshold 0.8

# Strict fuzzy similarity grouping (only very similar column names)
./file_metadata_finder --directory /path/to/data --output results.json --fuzzy-threshold 0.9

# Disable fuzzy similarity grouping
./file_metadata_finder --directory /path/to/data --output results.json --fuzzy-threshold 0

# Show all columns (unlimited)
./file_metadata_finder --directory /path/to/data --output results.json --max-columns 0

# Scan with default output file
./file_metadata_finder --directory /path/to/data
```

### Arguments

- `-d, --directory <PATH>`: Directory to scan (required)
- `-o, --output <OUTPUT_FILE>`: Output JSON file path (default: "output.json")
- `--disable-hash`: Disable CRC32 hash calculation (default: enabled for files ≤ 128KB)
- `--max-rows <NUMBER>`: Maximum rows to process for CSV/Excel files (default: 524,288)
- `--max-columns <NUMBER>`: Maximum columns to output for CSV/Excel files (0 = unlimited, default: 255)
- `--fuzzy-threshold <NUMBER>`: Fuzzy similarity threshold for column grouping (0.0-1.0, default: 0.8, 0 disables)

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
          "name": "large_dataset.csv",
          "created": "2025-10-03T13:40",
          "file_type": "csv",
          "crc32_hash": "a1b2c3d4",
          "csv_metadata": {
            "columns": ["ID", "Name", "Value"],
            "row_count": 1000,
            "column_similarity_hash": 288347173,
            "stopped_row_count_at": 1000
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
  ],
  "column_similarity_table": [
    {
      "hash": 288347173,
      "example_columns": ["ID", "Name", "Value"],
      "sources": [
        "/path/to/directory/large_dataset.csv",
        "/path/to/directory/data_[REDACTED].xlsx (Sheet1)"
      ]
    }
  ],
  "crc32_similarity_table": [
    {
      "hash": "08d6a686",
      "sources": [
        "/path/to/directory/document1.pdf",
        "/path/to/directory/document_copy.pdf"
      ]
    }
  ],
  "fuzzy_similarity_groups": [
    {
      "group_id": 0,
      "similarity_score": 0.8,
      "representative_columns": [
        "cust_id",
        "cust_name", 
        "customer_id",
        "customer_name",
        "user_id",
        "user_name"
      ],
      "sources": [
        "/path/to/directory/customers.csv",
        "/path/to/directory/users.csv"
      ]
    }
  ]
}
```

### Column Limiting

The tool supports intelligent column limiting for CSV and Excel files to handle large datasets:

#### Settings:
- **Default**: 255 columns maximum
- **Unlimited**: Use `--max-columns 0` to show all columns
- **Custom limit**: Use `--max-columns N` where N is your desired limit

#### Excel "Column1" Detection Rule:
To prevent Excel libraries from outputting invented column names, the tool automatically stops outputting columns if:
- Column position is 4 or greater (index ≥ 3), AND
- Column name is exactly "Column1"

This prevents output like: `["Real1", "Real2", "Real3", "Real4", "Column1", "Column2", "Column3"]`
And produces: `["Real1", "Real2", "Real3", "Real4"]`

#### Examples:
```bash
# Limit to 10 columns for large datasets
./file_metadata_finder --directory /data --max-columns 10

# Show all columns, but still apply "Column1" rule
./file_metadata_finder --directory /data --max-columns 0

# Use default limit of 255 columns
./file_metadata_finder --directory /data
```

### Row Limiting

To handle very large files efficiently, the tool supports configurable row limiting:

#### Settings:
- **Default**: 524,288 rows maximum (1024 * 512)
- **Custom limit**: Use `--max-rows N` where N is your desired limit
- **No limit**: Currently not supported (use a very large number if needed)

#### Behavior:
- When a file exceeds the row limit, processing stops at the limit
- The `stopped_row_count_at` field is added to indicate where processing stopped
- Row counting excludes header rows for accurate data row counts

#### Examples:
```bash
# Limit to 1000 rows for very large files
./file_metadata_finder --directory /data --max-rows 1000

# Use default limit of 524,288 rows
./file_metadata_finder --directory /data
```

### Column Similarity Table

The output includes a `column_similarity_table` that maps similarity hashes to their source files/sheets:

#### Purpose:
- Identifies datasets with identical column structures
- Helps find related or duplicate data schemas
- Supports data governance and catalog management

#### Features:
- Only shows hashes with multiple sources (similar datasets)
- Includes example columns from the first file with that structure
- For Excel files, includes sheet name in source description
- Sources are listed as file paths or "file (sheet)" format
- Table is sorted by hash value for consistency

#### Example Output:
```json
"column_similarity_table": [
  {
    "hash": 288347173,
    "example_columns": ["name", "age", "city"],
    "sources": [
      "./data1.csv",
      "./data2.csv",
      "./workbook.xlsx (Sheet1)"
    ]
  }
]
```

### CRC32 Similarity Table

The output includes a `crc32_similarity_table` that maps CRC32 hashes to files with identical content:

#### Purpose:
- Identifies files with identical content (exact duplicates)
- Helps detect duplicate files across different directories
- Supports deduplication and storage optimization
- Works across all file types that have CRC32 hashes calculated

#### Features:
- Only shows hashes with multiple sources (duplicate files)
- Only includes files ≤ 128KB (files with CRC32 hashes calculated)
- Sources are listed as file paths
- Table is sorted alphabetically by hash value for consistency

#### Example Output:
```json
"crc32_similarity_table": [
  {
    "hash": "08d6a686",
    "sources": [
      "./documents/report.pdf",
      "./backup/report_copy.pdf"
    ]
  },
  {
    "hash": "1a2b3c4d",
    "sources": [
      "./data/dataset1.csv",
      "./archive/dataset1_backup.csv",
      "./temp/dataset1_temp.csv"
    ]
  }
]
```

### Fuzzy Similarity Groups

The output includes a `fuzzy_similarity_groups` array that groups datasets with similar but not identical column structures:

#### Purpose:
- Identifies datasets with related column schemas using fuzzy string matching
- Groups files with similar naming conventions (e.g., "customer_id" vs "cust_id")
- Helps discover related datasets across different systems or time periods
- Supports schema evolution analysis and data integration planning

#### Features:
- Uses Jaro-Winkler similarity for fuzzy string matching of column names
- Configurable similarity threshold (0.0-1.0, default: 0.8)
- Only shows groups with multiple sources (similar datasets)
- Representative columns show union of all columns in the group
- Groups are assigned sequential IDs for reference

#### Similarity Algorithm:
1. **Individual Column Matching**: Uses Jaro-Winkler similarity (threshold: 0.8)
   - "customer_name" ↔ "cust_name" = high similarity
   - "user_id" ↔ "customer_id" = moderate similarity
2. **Set Similarity**: Modified Jaccard similarity using fuzzy matches
3. **Clustering**: Groups datasets that exceed the overall threshold

#### Threshold Guidelines:
- **0.9+**: Very strict (only minor abbreviations: "cust_id" ↔ "customer_id")
- **0.8**: Default (moderate similarity: related concepts with different naming)
- **0.6-0.7**: Lenient (broader concept matching: "id" fields, "name" fields)
- **0.0**: Disabled (no fuzzy grouping performed)

#### Example Output:
```json
"fuzzy_similarity_groups": [
  {
    "group_id": 0,
    "similarity_score": 0.8,
    "representative_columns": [
      "cust_id", "cust_name", "customer_id", "customer_name"
    ],
    "sources": [
      "./old_system/customers.csv",
      "./new_system/cust_data.csv"
    ]
  }
]
```

### Field Descriptions

- **`scan_directory`**: Absolute path of the directory that was scanned
- **`directories`**: Array of directories containing matching files
- **`column_similarity_table`**: Array of similarity hash mappings showing datasets with identical column structures
- **`crc32_similarity_table`**: Array of CRC32 hash mappings showing files with identical content
- **`fuzzy_similarity_groups`**: Array of groups containing datasets with similar but not identical column structures
- **`created`**: File creation timestamp in simplified format (YYYY-MM-DDTHH:MM)
- **`crc32_hash`**: Present for files ≤ 128KB (default behavior). 8-character hexadecimal CRC32 hash
- **`file_size`**: Present for files > 128KB or when `--disable-hash` is used. Size in bytes
- **`column_similarity_hash`**: Present for CSV and Excel files. CRC32 hash of processed column names (lowercase, alphanumeric only, sorted) to identify structurally similar datasets
- **`stopped_row_count_at`**: Present when row limiting is applied. Indicates the number of rows processed before stopping
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
