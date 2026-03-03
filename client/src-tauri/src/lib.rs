use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

// ── Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HexRow {
    pub offset: String,
    pub hex: Vec<String>,
    pub ascii: String,
}

#[derive(Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub file_type: String,
}

#[derive(Serialize)]
pub struct ReadResult {
    pub rows: Vec<HexRow>,
    pub total_bytes: u64,
}

#[derive(Serialize)]
pub struct StringMatch {
    pub offset: u64,
    pub value: String,
}

// ── Tauri commands ──────────────────────────────────────────────────

/// Open a file and return its metadata + detected type.
#[tauri::command]
fn open_file(path: String) -> Result<FileInfo, String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    let file_type = detect_file_type(&path);
    Ok(FileInfo {
        path,
        size: meta.len(),
        file_type,
    })
}

/// Read a range of bytes from a file and return hex-dump rows.
/// `start` and `end` are inclusive byte offsets (negative = from end).
#[tauri::command]
fn read_bytes(path: String, start: i64, end: i64) -> Result<ReadResult, String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    let size = meta.len() as i64;

    let s = resolve_offset(start, size);
    let e = if end == i64::MAX {
        (size - 1).max(0) as u64
    } else {
        resolve_offset(end, size)
    };

    if s > e || s >= size as u64 {
        return Ok(ReadResult {
            rows: vec![],
            total_bytes: size as u64,
        });
    }

    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(s)).map_err(|e| e.to_string())?;
    let len = (e - s + 1) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;

    let rows = buf
        .chunks(16)
        .enumerate()
        .map(|(i, chunk)| {
            let offset = format!("{:08X}", s as usize + i * 16);
            let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
            let ascii: String = chunk
                .iter()
                .map(|&b| if (0x20..=0x7E).contains(&b) { b as char } else { '.' })
                .collect();
            HexRow { offset, hex, ascii }
        })
        .collect();

    Ok(ReadResult {
        rows,
        total_bytes: size as u64,
    })
}

/// Get the size of a file in bytes.
#[tauri::command]
fn get_file_size(path: String) -> Result<u64, String> {
    fs::metadata(&path).map(|m| m.len()).map_err(|e| e.to_string())
}

/// Get the detected file type by magic bytes.
#[tauri::command]
fn get_file_type(path: String) -> Result<String, String> {
    Ok(detect_file_type(&path))
}

/// Write (overwrite) data at a given offset. Supports:
///  - single offset overwrite
///  - range overwrite (start..end inclusive)
///  - `append_zero_past_eof`: zero-pad if offset > current size
///  - `reverse`: reverse data bytes before writing
///  - `data_file`: if set, read data from this file path instead of `data` string
///  - `data_file_offset`: byte offset to start reading from within the data file
#[tauri::command]
fn write_overwrite(
    path: String,
    offset: i64,
    end_offset: Option<i64>,
    data: String,
    data_file: Option<String>,
    data_file_offset: Option<u64>,
    append_zero_past_eof: bool,
    reverse: bool,
) -> Result<String, String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    let size = meta.len() as i64;

    let mut bytes: Vec<u8> = if let Some(ref file_path) = data_file {
        let all = fs::read(file_path).map_err(|e| format!("Failed to read data file '{}': {}", file_path, e))?;
        if let Some(off) = data_file_offset {
            let off = off as usize;
            if off >= all.len() { Vec::new() } else { all[off..].to_vec() }
        } else {
            all
        }
    } else {
        data.into_bytes()
    };
    if reverse {
        bytes.reverse();
    }

    let start = resolve_offset(offset, size);

    if let Some(end_raw) = end_offset {
        // Range overwrite
        let end = resolve_offset(end_raw, size);
        let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
        let range_len = (hi - lo + 1) as usize;
        if bytes.len() != range_len {
            return Err(format!(
                "Data length ({}) must match range length ({}) for overwrite",
                bytes.len(),
                range_len
            ));
        }
        // Descending range: reverse data
        if start > end {
            bytes.reverse();
        }
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(lo)).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
    } else {
        // Single offset overwrite
        if start as i64 > size && !append_zero_past_eof {
            return Err("Offset past EOF; use append_zero_past_eof flag".into());
        }
        if start as i64 > size && append_zero_past_eof {
            // Zero-pad
            let mut file = OpenOptions::new()
                .write(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
            let pad = vec![0u8; (start as i64 - size) as usize];
            file.write_all(&pad).map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
        } else {
            let write_len = bytes.len().min((size as u64 - start) as usize);
            let mut file = OpenOptions::new()
                .write(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
            file.write_all(&bytes[..write_len]).map_err(|e| e.to_string())?;
        }
    }

    Ok("OK".into())
}

/// Insert (splice) data at a given offset, shifting subsequent bytes forward.
/// If `data_file` is set, read data from that file path instead of `data` string.
/// If `data_file_offset` is set, start reading from that byte offset within the data file.
#[tauri::command]
fn write_insert(
    path: String,
    offset: i64,
    data: String,
    data_file: Option<String>,
    data_file_offset: Option<u64>,
    reverse: bool,
) -> Result<String, String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    let size = meta.len() as i64;
    let pos = resolve_offset(offset, size);

    let mut bytes: Vec<u8> = if let Some(ref file_path) = data_file {
        let all = fs::read(file_path).map_err(|e| format!("Failed to read data file '{}': {}", file_path, e))?;
        if let Some(off) = data_file_offset {
            let off = off as usize;
            if off >= all.len() { Vec::new() } else { all[off..].to_vec() }
        } else {
            all
        }
    } else {
        data.into_bytes()
    };
    if reverse {
        bytes.reverse();
    }

    let mut contents = fs::read(&path).map_err(|e| e.to_string())?;
    let insert_at = (pos as usize).min(contents.len());
    for (i, b) in bytes.iter().enumerate() {
        contents.insert(insert_at + i, *b);
    }
    fs::write(&path, &contents).map_err(|e| e.to_string())?;

    Ok("OK".into())
}

/// Copy a file from src to dest.
#[tauri::command]
fn copy_file(src: String, dest: String) -> Result<String, String> {
    fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    Ok(format!("Copied {} to {}", src, dest))
}

/// Scan a file for printable ASCII string sequences.
/// `min_length` sets the minimum run length (default 4).
#[tauri::command]
fn scan_strings(path: String, min_length: Option<usize>) -> Result<Vec<StringMatch>, String> {
    let min = min_length.unwrap_or(4);
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    let mut results: Vec<StringMatch> = Vec::new();
    let mut run_start: Option<usize> = None;
    let mut run_len = 0usize;

    for (i, &b) in data.iter().enumerate() {
        let printable = (0x20..=0x7E).contains(&b) || b == 0x09 || b == 0x0A || b == 0x0D;
        if printable {
            if run_start.is_none() {
                run_start = Some(i);
                run_len = 0;
            }
            run_len += 1;
        } else {
            if let Some(start) = run_start {
                if run_len >= min {
                    let s = String::from_utf8_lossy(&data[start..start + run_len]).into_owned();
                    results.push(StringMatch { offset: start as u64, value: s });
                }
            }
            run_start = None;
            run_len = 0;
        }
    }
    if let Some(start) = run_start {
        if run_len >= min {
            let s = String::from_utf8_lossy(&data[start..start + run_len]).into_owned();
            results.push(StringMatch { offset: start as u64, value: s });
        }
    }
    Ok(results)
}

// ── Helpers ─────────────────────────────────────────────────────────

fn resolve_offset(offset: i64, file_size: i64) -> u64 {
    if offset < 0 {
        (file_size + offset).max(0) as u64
    } else {
        offset as u64
    }
}

fn detect_file_type(path: &str) -> String {
    let mut buf = [0u8; 8];
    let Ok(mut f) = File::open(path) else {
        return "unknown".into();
    };
    let n = f.read(&mut buf).unwrap_or(0);
    if n < 2 {
        return "unknown".into();
    }

    // JPG: FF D8
    if buf[0] == 0xFF && buf[1] == 0xD8 {
        return "jpg".into();
    }
    // PNG: 89 50 4E 47
    if n >= 4 && buf[..4] == [0x89, 0x50, 0x4E, 0x47] {
        return "png".into();
    }
    // GIF: 47 49 46 38
    if n >= 4 && buf[..4] == [0x47, 0x49, 0x46, 0x38] {
        return "gif".into();
    }
    // BMP: 42 4D
    if buf[0] == 0x42 && buf[1] == 0x4D {
        return "bmp".into();
    }
    // EXE/PE: 4D 5A
    if buf[0] == 0x4D && buf[1] == 0x5A {
        return "exe".into();
    }
    // Java class: CA FE BA BE
    if n >= 4 && buf[..4] == [0xCA, 0xFE, 0xBA, 0xBE] {
        return "class".into();
    }
    // PDF: 25 50 44 46
    if n >= 4 && buf[..4] == [0x25, 0x50, 0x44, 0x46] {
        return "pdf".into();
    }

    "unknown".into()
}

// ── Tauri app entry ─────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_file,
            read_bytes,
            get_file_size,
            get_file_type,
            write_overwrite,
            write_insert,
            copy_file,
            scan_strings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
