use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Data, Range, Reader};
use chrono::{DateTime, Utc};
use clap::Parser;
use crc32fast::Hasher;
use csv::ReaderBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use strsim::jaro_winkler;
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

    /// Disable CRC32 hash calculation for files <= 128KB (hash is enabled by default)
    #[arg(long, default_value_t = false)]
    disable_hash: bool,

    /// Maximum number of rows to process for CSV and Excel files (default: 524288)
    #[arg(long, default_value_t = 524288)]
    max_rows: usize,

    /// Fuzzy similarity threshold for column grouping (0.0-1.0, default: 0.8, 0 disables)
    #[arg(long, default_value_t = 0.8)]
    fuzzy_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimilarityHashEntry {
    hash: u32,
    example_columns: Vec<String>,
    sources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Crc32HashEntry {
    hash: String,
    sources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FuzzySimilarityGroup {
    group_id: usize,
    similarity_score: f64,
    representative_columns: Vec<String>,
    sources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanResult {
    scan_directory: String,
    directories: Vec<DirectoryEntry>,
    column_similarity_table: Vec<SimilarityHashEntry>,
    crc32_similarity_table: Vec<Crc32HashEntry>,
    fuzzy_similarity_groups: Vec<FuzzySimilarityGroup>,
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
    file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crc32_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    csv_metadata: Option<CsvMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excel_metadata: Option<ExcelMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CsvMetadata {
    columns: Vec<String>,
    row_count: usize,
    column_similarity_hash: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped_row_count_at: Option<usize>,
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
    column_similarity_hash: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped_row_count_at: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.directory.exists() {
        anyhow::bail!("Directory does not exist: {:?}", args.directory);
    }

    println!("Scanning directory: {:?}", args.directory);
    println!("Output file: {:?}", args.output);

    let entries = scan_directory(&args.directory, !args.disable_hash, args.max_rows)?;

    // Build column similarity table
    let similarity_table = build_similarity_table(&entries);

    // Build CRC32 similarity table
    let crc32_table = build_crc32_table(&entries);

    // Build fuzzy similarity groups
    let fuzzy_groups = if args.fuzzy_threshold > 0.0 {
        build_fuzzy_similarity_groups(&entries, args.fuzzy_threshold)
    } else {
        Vec::new()
    };

    // Create the top-level result with absolute path
    let scan_result = ScanResult {
        scan_directory: args.directory.canonicalize()
            .unwrap_or_else(|_| args.directory.clone())
            .display()
            .to_string(),
        directories: entries,
        column_similarity_table: similarity_table,
        crc32_similarity_table: crc32_table,
        fuzzy_similarity_groups: fuzzy_groups,
    };

    // Write JSON output
    let json = serde_json::to_string_pretty(&scan_result)?;
    let mut file = File::create(&args.output)
        .context(format!("Failed to create output file: {:?}", args.output))?;
    file.write_all(json.as_bytes())?;

    println!(
        "\nCompleted! Found {} directories with files.",
        scan_result.directories.len()
    );
    println!("Output written to: {:?}", args.output);

    Ok(())
}

fn scan_directory(path: &Path, enable_hash: bool, max_rows: usize) -> Result<Vec<DirectoryEntry>> {
    let mut dir_map: HashMap<PathBuf, Vec<FileDetails>> = HashMap::new();

    // First pass: count files for progress bar
    let file_count = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_file_type(e.path()))
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
        .filter(|e| is_supported_file_type(e.path()))
    {
        let file_path = entry.path();
        let parent_dir = file_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf();

        pb.set_message(format!("Processing: {}", file_path.display()));

        if let Ok(file_details) = process_file(file_path, enable_hash, max_rows) {
            dir_map.entry(parent_dir).or_default().push(file_details);
        }

        pb.inc(1);
    }

    pb.finish_with_message("Processing complete");

    // Convert to output format, excluding directories with no files
    let mut entries: Vec<DirectoryEntry> = dir_map
        .into_iter()
        .filter(|(_, files)| !files.is_empty())
        .map(|(path, files)| DirectoryEntry {
            path: redact_nhs_numbers(&path.display().to_string()),
            files,
        })
        .collect();

    // Sort by path for consistent output
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

fn build_similarity_table(directories: &[DirectoryEntry]) -> Vec<SimilarityHashEntry> {
    let mut hash_map: HashMap<u32, (Vec<String>, Vec<String>)> = HashMap::new(); // (sources, example_columns)

    for dir_entry in directories {
        for file_details in &dir_entry.files {
            let file_path = format!("{}/{}", dir_entry.path, file_details.name);

            // Collect CSV similarity hashes
            if let Some(csv_meta) = &file_details.csv_metadata {
                let entry = hash_map
                    .entry(csv_meta.column_similarity_hash)
                    .or_insert_with(|| (Vec::new(), csv_meta.columns.clone()));
                entry.0.push(file_path.clone());
            }

            // Collect Excel similarity hashes (per sheet)
            if let Some(excel_meta) = &file_details.excel_metadata {
                for sheet in &excel_meta.sheets {
                    let sheet_source = format!("{} ({})", file_path, sheet.sheet_name);
                    let entry = hash_map
                        .entry(sheet.column_similarity_hash)
                        .or_insert_with(|| (Vec::new(), sheet.columns.clone()));
                    entry.0.push(sheet_source);
                }
            }
        }
    }

    // Convert to sorted vector, only including hashes with multiple sources
    let mut similarity_table: Vec<SimilarityHashEntry> = hash_map
        .into_iter()
        .filter(|(_, (sources, _))| sources.len() > 1)  // Only show hashes with multiple sources
        .map(|(hash, (sources, example_columns))| SimilarityHashEntry {
            hash,
            example_columns,
            sources
        })
        .collect();

    similarity_table.sort_by_key(|entry| entry.hash);
    similarity_table
}

fn build_crc32_table(directories: &[DirectoryEntry]) -> Vec<Crc32HashEntry> {
    let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();

    for dir_entry in directories {
        for file_details in &dir_entry.files {
            let file_path = format!("{}/{}", dir_entry.path, file_details.name);

            // Collect CRC32 hashes (only for files that have them)
            if let Some(crc32_hash) = &file_details.crc32_hash {
                hash_map
                    .entry(crc32_hash.clone())
                    .or_default()
                    .push(file_path);
            }
        }
    }

    // Convert to sorted vector, only including hashes with multiple sources
    let mut crc32_table: Vec<Crc32HashEntry> = hash_map
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)  // Only show hashes with multiple sources
        .map(|(hash, sources)| Crc32HashEntry { hash, sources })
        .collect();

    crc32_table.sort_by(|a, b| a.hash.cmp(&b.hash));
    crc32_table
}

fn build_fuzzy_similarity_groups(directories: &[DirectoryEntry], threshold: f64) -> Vec<FuzzySimilarityGroup> {
    // Collect all column sets with their sources
    let mut column_sets: Vec<(Vec<String>, String)> = Vec::new();

    for dir_entry in directories {
        for file_details in &dir_entry.files {
            let file_path = format!("{}/{}", dir_entry.path, file_details.name);

            // Collect CSV columns
            if let Some(csv_meta) = &file_details.csv_metadata {
                column_sets.push((csv_meta.columns.clone(), file_path.clone()));
            }

            // Collect Excel columns (per sheet)
            if let Some(excel_meta) = &file_details.excel_metadata {
                for sheet in &excel_meta.sheets {
                    let sheet_source = format!("{} ({})", file_path, sheet.sheet_name);
                    column_sets.push((sheet.columns.clone(), sheet_source));
                }
            }
        }
    }

    if column_sets.len() < 2 {
        return Vec::new();
    }

    // Group similar column sets using clustering approach
    let mut groups: Vec<FuzzySimilarityGroup> = Vec::new();
    let mut used_indices: Vec<bool> = vec![false; column_sets.len()];
    let mut group_id = 0;

    for i in 0..column_sets.len() {
        if used_indices[i] {
            continue;
        }

        let mut group_sources = vec![column_sets[i].1.clone()];
        let mut group_columns = column_sets[i].0.clone();
        used_indices[i] = true;

        // Find similar column sets
        for j in (i + 1)..column_sets.len() {
            if used_indices[j] {
                continue;
            }

            let similarity = calculate_column_set_similarity(&column_sets[i].0, &column_sets[j].0);
            if similarity >= threshold {
                group_sources.push(column_sets[j].1.clone());
                // Merge columns (union for representative set)
                for col in &column_sets[j].0 {
                    if !group_columns.contains(col) {
                        group_columns.push(col.clone());
                    }
                }
                used_indices[j] = true;
            }
        }

        // Only create groups with multiple sources
        if group_sources.len() > 1 {
            group_columns.sort();
            groups.push(FuzzySimilarityGroup {
                group_id,
                similarity_score: threshold,
                representative_columns: group_columns,
                sources: group_sources,
            });
            group_id += 1;
        }
    }

    groups
}

fn calculate_column_set_similarity(set1: &[String], set2: &[String]) -> f64 {
    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }
    if set1.is_empty() || set2.is_empty() {
        return 0.0;
    }

    // Use Jaccard similarity as base, enhanced with fuzzy string matching
    let mut matches = 0;

    for col1 in set1 {
        let mut best_match = 0.0;
        for col2 in set2 {
            let similarity = jaro_winkler(col1, col2);
            if similarity > best_match {
                best_match = similarity;
            }
        }
        if best_match > 0.8 {  // Individual column similarity threshold
            matches += 1;
        }
    }

    // Calculate fuzzy Jaccard: matches / union_size
    let union_size = set1.len() + set2.len() - matches;
    if union_size == 0 {
        1.0
    } else {
        matches as f64 / union_size as f64
    }
}

fn is_supported_file_type(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "csv" | "xlsx" | "xls" | "xlsm" | "xlsb" | "pdf" | "docx" | "eml"
        )
    } else {
        false
    }
}

fn process_file(path: &Path, enable_hash: bool, max_rows: usize) -> Result<FileDetails> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let redacted_name = redact_nhs_numbers(&file_name);

    let created = get_creation_time(path)?;
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();

    // Calculate hash for files <= 128KB, otherwise just store file size
    const MAX_HASH_SIZE: u64 = 128 * 1024; // 128KB
    let (hash_value, size_value) = if enable_hash && file_size <= MAX_HASH_SIZE {
        (Some(calculate_crc32(path)?), None)
    } else {
        (None, Some(file_size))
    };

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut file_details = FileDetails {
        name: redacted_name,
        created,
        file_type: None,
        file_size: size_value,
        crc32_hash: hash_value,
        csv_metadata: None,
        excel_metadata: None,
    };

    match extension.as_str() {
        "csv" => {
            file_details.file_type = Some("csv".to_string());
            if let Ok(csv_meta) = extract_csv_metadata(path, max_rows) {
                file_details.csv_metadata = Some(csv_meta);
            }
        }
        "xlsx" | "xls" | "xlsm" | "xlsb" => {
            file_details.file_type = Some("excel".to_string());
            if let Ok(excel_meta) = extract_excel_metadata(path, max_rows) {
                file_details.excel_metadata = Some(excel_meta);
            }
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
    Ok(datetime.format("%Y-%m-%dT%H:%M").to_string())
}

fn calculate_crc32(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buffer = [0; 8192]; // 8KB buffer for reading

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:08x}", hasher.finalize()))
}

fn calculate_column_similarity_hash(columns: &[String]) -> u32 {
    // Process column names: lowercase, remove non-alphanumeric, filter empty, sort
    let mut processed_columns: Vec<String> = columns
        .iter()
        .map(|col| {
            col.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|col| !col.is_empty())
        .collect();

    processed_columns.sort();
    let concatenated = processed_columns.join(",");

    let mut hasher = Hasher::new();
    hasher.update(concatenated.as_bytes());
    hasher.finalize()
}

fn extract_csv_metadata(path: &Path, max_rows: usize) -> Result<CsvMetadata> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;

    let headers = reader.headers()?.clone();
    let columns: Vec<String> = headers.iter().map(redact_nhs_numbers).collect();

    let mut row_count = 0;
    let mut stopped_at = None;

    for record in reader.records() {
        if record.is_ok() {
            row_count += 1;
            if row_count >= max_rows {
                stopped_at = Some(row_count);
                break;
            }
        }
    }

    let similarity_hash = calculate_column_similarity_hash(&columns);

    Ok(CsvMetadata {
        columns,
        row_count,
        column_similarity_hash: similarity_hash,
        stopped_row_count_at: stopped_at,
    })
}

fn extract_excel_metadata(path: &Path, max_rows: usize) -> Result<ExcelMetadata> {
    let mut workbook = open_workbook_auto(path)?;
    let mut sheets = Vec::new();

    for sheet_name in workbook.sheet_names().to_vec() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let (columns, header_row_idx) = extract_excel_columns_with_header_row(&range);

            // Calculate actual data rows (excluding header)
            let total_data_rows = if range.height() > header_row_idx + 1 {
                range.height() - header_row_idx - 1
            } else {
                0
            };

            let (row_count, stopped_at) = if total_data_rows > max_rows {
                (max_rows, Some(max_rows))
            } else {
                (total_data_rows, None)
            };

            let similarity_hash = calculate_column_similarity_hash(&columns);

            sheets.push(SheetMetadata {
                sheet_name: redact_nhs_numbers(&sheet_name),
                columns,
                row_count,
                column_similarity_hash: similarity_hash,
                stopped_row_count_at: stopped_at,
            });
        }
    }

    Ok(ExcelMetadata { sheets })
}

fn extract_excel_columns_with_header_row(range: &Range<Data>) -> (Vec<String>, usize) {
    // Smart matching: search first 5 rows for headers
    let max_rows = 5.min(range.height());
    let mut best_headers: Vec<String> = Vec::new();
    let mut best_score = 0;
    let mut best_row_idx = 0;

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
            best_row_idx = row_idx;
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
        best_row_idx = 0;
    }

    (best_headers, best_row_idx)
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
