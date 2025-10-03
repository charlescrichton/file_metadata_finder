#!/bin/bash
# Integration test script for file_metadata_finder

set -e

echo "Building the project..."
cargo build --release

echo -e "\nCreating test data..."
TEST_DIR=$(mktemp -d)
mkdir -p "$TEST_DIR/subdir"

# Create test CSV
cat > "$TEST_DIR/patient_1234567890.csv" << 'EOF'
ID,Name,NHS Number,Age
1,John Doe,9876543210,45
2,Jane Smith,111 222 3333,32
EOF

# Create test subdirectory CSV
cat > "$TEST_DIR/subdir/data.csv" << 'EOF'
Column1,Column2
Value1,Value2
Value3,Value4
EOF

# Create dummy files
touch "$TEST_DIR/report.pdf"
touch "$TEST_DIR/document_5555555555.docx"
touch "$TEST_DIR/subdir/email.eml"

OUTPUT_FILE=$(mktemp)

echo -e "\nRunning file_metadata_finder..."
./target/release/file_metadata_finder --directory "$TEST_DIR" --output "$OUTPUT_FILE"

echo -e "\nVerifying output..."

# Check that output file was created
if [ ! -f "$OUTPUT_FILE" ]; then
    echo "ERROR: Output file was not created"
    exit 1
fi

# Check JSON is valid
if ! python3 -m json.tool "$OUTPUT_FILE" > /dev/null 2>&1; then
    echo "ERROR: Output is not valid JSON"
    exit 1
fi

# Check for NHS redaction in filename
if grep -q "patient_1234567890" "$OUTPUT_FILE"; then
    echo "ERROR: NHS number in filename was not redacted"
    exit 1
fi

if ! grep -q "patient_\[REDACTED\]" "$OUTPUT_FILE"; then
    echo "ERROR: Expected redacted filename not found"
    exit 1
fi

# Check for CSV metadata
if ! grep -q '"csv_metadata"' "$OUTPUT_FILE"; then
    echo "ERROR: CSV metadata not found in output"
    exit 1
fi

# Check for file types
if ! grep -q '"file_type": "pdf"' "$OUTPUT_FILE"; then
    echo "ERROR: PDF file type not found"
    exit 1
fi

if ! grep -q '"file_type": "docx"' "$OUTPUT_FILE"; then
    echo "ERROR: DOCX file type not found"
    exit 1
fi

if ! grep -q '"file_type": "eml"' "$OUTPUT_FILE"; then
    echo "ERROR: EML file type not found"
    exit 1
fi

echo -e "\n✅ All tests passed!"
echo "Test directory: $TEST_DIR"
echo "Output file: $OUTPUT_FILE"

# Cleanup
rm -rf "$TEST_DIR"
rm "$OUTPUT_FILE"

echo "Cleanup complete."
