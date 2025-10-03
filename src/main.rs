use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Data, Range, Reader};
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::ReaderBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan
    #[arg(short, long)]
    directory: PathBuf,

    /// Output JSON file path
    #[arg(short, long, default_value = "output.json")]
    output: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryEntry {
    path: String,
    files: Vec<FileDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileDetails {
    name: String,
    created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    csv_metadata: Option<CsvMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excel_metadata: Option<ExcelMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CsvMetadata {
    columns: Vec<String>,
    row_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExcelMetadata {
    sheets: Vec<SheetMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SheetMetadata {
    sheet_name: String,
    columns: Vec<String>,
    row_count: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.directory.exists() {
        anyhow::bail!("Directory does not exist: {:?}", args.directory);
    }

    println!("Scanning directory: {:?}", args.directory);
    println!("Output file: {:?}", args.output);

    let entries = scan_directory(&args.directory)?;

    // Write JSON output
    let json = serde_json::to_string_pretty(&entries)?;
    let mut file = File::create(&args.output)
        .context(format!("Failed to create output file: {:?}", args.output))?;
    file.write_all(json.as_bytes())?;

    println!(
        "\nCompleted! Found {} directories with files.",
        entries.len()
    );
    println!("Output written to: {:?}", args.output);

    Ok(())
}

fn scan_directory(path: &Path) -> Result<Vec<DirectoryEntry>> {
    let mut dir_map: HashMap<PathBuf, Vec<FileDetails>> = HashMap::new();

    // First pass: count files for progress bar
    let file_count = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    println!("Found {} files to process", file_count);

    let pb = ProgressBar::new(file_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Second pass: process files
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        let parent_dir = file_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf();

        pb.set_message(format!("Processing: {}", file_path.display()));

        if let Ok(file_details) = process_file(file_path) {
            dir_map.entry(parent_dir).or_default().push(file_details);
        }

        pb.inc(1);
    }

    pb.finish_with_message("Processing complete");

    // Convert to output format
    let mut entries: Vec<DirectoryEntry> = dir_map
        .into_iter()
        .map(|(path, files)| DirectoryEntry {
            path: redact_nhs_numbers(&path.display().to_string()),
            files,
        })
        .collect();

    // Sort by path for consistent output
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

fn process_file(path: &Path) -> Result<FileDetails> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let redacted_name = redact_nhs_numbers(&file_name);

    let created = get_creation_time(path)?;

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut file_details = FileDetails {
        name: redacted_name,
        created,
        file_type: None,
        csv_metadata: None,
        excel_metadata: None,
    };

    match extension.as_str() {
        "csv" => {
            file_details.file_type = Some("csv".to_string());
            file_details.csv_metadata = extract_csv_metadata(path).ok();
        }
        "xlsx" | "xls" | "xlsm" | "xlsb" => {
            file_details.file_type = Some("excel".to_string());
            file_details.excel_metadata = extract_excel_metadata(path).ok();
        }
        "pdf" => {
            file_details.file_type = Some("pdf".to_string());
        }
        "docx" => {
            file_details.file_type = Some("docx".to_string());
        }
        "eml" => {
            file_details.file_type = Some("eml".to_string());
        }
        _ => {}
    }

    Ok(file_details)
}

fn get_creation_time(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    let created = metadata
        .created()
        .or_else(|_| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let datetime: DateTime<Utc> = created.into();
    Ok(datetime.to_rfc3339())
}

fn extract_csv_metadata(path: &Path) -> Result<CsvMetadata> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;

    let headers = reader.headers()?.clone();
    let columns: Vec<String> = headers.iter().map(redact_nhs_numbers).collect();

    let row_count = reader.records().count();

    Ok(CsvMetadata { columns, row_count })
}

fn extract_excel_metadata(path: &Path) -> Result<ExcelMetadata> {
    let mut workbook = open_workbook_auto(path)?;
    let mut sheets = Vec::new();

    for sheet_name in workbook.sheet_names().to_vec() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let columns = extract_excel_columns(&range);
            let row_count = range.height();

            sheets.push(SheetMetadata {
                sheet_name: redact_nhs_numbers(&sheet_name),
                columns,
                row_count,
            });
        }
    }

    Ok(ExcelMetadata { sheets })
}

fn extract_excel_columns(range: &Range<Data>) -> Vec<String> {
    // Smart matching: search first 5 rows for headers
    let max_rows = 5.min(range.height());
    let mut best_headers: Vec<String> = Vec::new();
    let mut best_score = 0;

    for row_idx in 0..max_rows {
        let mut headers = Vec::new();
        let mut score = 0;

        if let Some(row) = range.rows().nth(row_idx) {
            for cell in row {
                let cell_str = cell.to_string().trim().to_string();
                if !cell_str.is_empty() {
                    headers.push(redact_nhs_numbers(&cell_str));
                    // Heuristic: prefer rows with more non-empty, non-numeric cells
                    if !cell_str.chars().all(|c| c.is_numeric() || c == '.') {
                        score += 1;
                    }
                }
            }
        }

        if score > best_score && !headers.is_empty() {
            best_score = score;
            best_headers = headers;
        }
    }

    // If no good headers found, use first row
    if best_headers.is_empty() {
        if let Some(first_row) = range.rows().next() {
            best_headers = first_row
                .iter()
                .map(|cell| redact_nhs_numbers(cell.to_string().trim()))
                .collect();
        }
    }

    best_headers
}

fn redact_nhs_numbers(text: &str) -> String {
    // Pattern 1: 10 consecutive digits
    // Matches 10 digits that are not part of a longer sequence
    let re_ten_digits = Regex::new(r"\d{10}").unwrap();
    // Pattern 2: nnn nnn nnnn format
    let re_spaced = Regex::new(r"\d{3}\s+\d{3}\s+\d{4}").unwrap();

    let mut result = text.to_string();

    // First replace spaced format (more specific)
    result = re_spaced.replace_all(&result, "[REDACTED]").to_string();

    // Then replace 10 consecutive digits
    // But filter out cases where they're part of longer numbers
    let mut final_result = String::new();
    let mut last_end = 0;

    for mat in re_ten_digits.find_iter(&result) {
        let start = mat.start();
        let end = mat.end();

        // Check if this digit sequence is isolated (not part of longer number)
        let before_ok = start == 0 || !result.as_bytes()[start - 1].is_ascii_digit();
        let after_ok = end >= result.len() || !result.as_bytes()[end].is_ascii_digit();

        final_result.push_str(&result[last_end..start]);

        if before_ok && after_ok {
            final_result.push_str("[REDACTED]");
        } else {
            final_result.push_str(mat.as_str());
        }

        last_end = end;
    }

    final_result.push_str(&result[last_end..]);
    final_result
}
