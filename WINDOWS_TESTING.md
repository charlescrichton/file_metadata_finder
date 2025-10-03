# Windows Path Testing Guide

This document outlines how to test the Windows executable to ensure proper path handling.

## Testing the Windows Executable

1. **Copy the executable to Windows**:
   ```
   target/x86_64-pc-windows-gnu/release/file_metadata_finder.exe
   ```

2. **Test with various Windows path formats**:

   ### PowerShell 7 Examples:
   ```powershell
   # Test with C: drive
   .\file_metadata_finder.exe --directory "C:\Users\$env:USERNAME\Documents" --output "test_results.json"
   
   # Test with CRC32 hash calculation enabled
   .\file_metadata_finder.exe --directory "C:\Users\$env:USERNAME\Documents" --output "hash_results.json" --enable-hash
   
   # Test with different drive letters
   .\file_metadata_finder.exe --directory "D:\MyData" --output "d_drive_scan.json"
   
   # Test with UNC paths (if available)
   .\file_metadata_finder.exe --directory "\\server\share\folder" --output "unc_results.json"
   
   # Test with spaces in paths
   .\file_metadata_finder.exe --directory "C:\Program Files\Test Folder" --output "spaces_test.json"
   ```

   ### Command Prompt Examples:
   ```cmd
   REM Test basic functionality
   file_metadata_finder.exe --directory "C:\Users\%USERNAME%\Documents" --output "cmd_test.json"
   
   REM Test with hash calculation
   file_metadata_finder.exe --directory "C:\Users\%USERNAME%\Documents" --output "cmd_hash_test.json" --enable-hash
   
   REM Test with relative paths
   file_metadata_finder.exe --directory ".\test_data" --output "relative_test.json"
   ```

## Expected Behavior

- **Path Recognition**: Should properly handle Windows drive letters (C:\, D:\, etc.)
- **Backslash Handling**: Should correctly interpret Windows backslash path separators
- **Unicode Support**: Should handle Unicode characters in file names and paths
- **Long Paths**: Should work with paths longer than 260 characters (if Windows LongPathsEnabled)
- **Case Insensitivity**: Should work with Windows' case-insensitive file system

## Verification Points

1. **JSON Output**: Paths in the output JSON should use forward slashes (/) as per JSON standard
2. **File Detection**: Should correctly identify and process CSV, Excel, PDF, DOCX, and EML files
3. **Progress Bar**: Should display correctly in PowerShell and Command Prompt
4. **Error Handling**: Should provide clear error messages for invalid paths
5. **NHS Redaction**: Should work correctly on Windows file paths and names
6. **Hash Calculation**: When `--enable-hash` is used:
   - Files ≤ 128KB should have `crc32_hash` field with 8-character hex value
   - Files > 128KB should have `file_size` field with byte count
7. **Size Reporting**: When `--enable-hash` is not used, all files should have `file_size` field

## Common Windows-Specific Issues to Watch For

- **Drive Letters**: Ensure C:\, D:\, etc. are handled correctly
- **UNC Paths**: Network paths should work if accessible
- **Reserved Names**: Should handle Windows reserved names (CON, PRN, AUX, etc.)
- **Path Length**: Should handle both short and long Windows paths
- **Permissions**: Should handle permission denied errors gracefully

## Sample Test Data Structure

Create a test directory structure like:
```
C:\TestData\
├── files_with_spaces\
│   ├── test file.csv
│   └── document with spaces.pdf
├── unicode_files\
│   ├── résumé.xlsx
│   └── файл.docx
└── nhs_test\
    ├── patient_1234567890.csv
    └── data_123 456 7890.xlsx
```

This will test path handling, space handling, Unicode support, and NHS number redaction.