use clap::{Command, Arg, ArgAction};
use std::fs;
use std::fs::File;
use std::io::Write;
use binrw_cli::{read_range, read_range_i64_negative_start, read_bytes};

/// Parse a string as i64, treating a "0x" / "0X" / "-0x" prefix as hexadecimal.
fn parse_int_auto_i64(s: &str) -> Result<i64, String> {
    let trimmed = s.trim();
    if trimmed.starts_with("-0x") || trimmed.starts_with("-0X") {
        i64::from_str_radix(&trimmed[3..], 16)
            .map(|v| -v)
            .map_err(|e| format!("Invalid hex integer '{}': {}", s, e))
    } else if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        i64::from_str_radix(&trimmed[2..], 16)
            .map_err(|e| format!("Invalid hex integer '{}': {}", s, e))
    } else {
        trimmed.parse::<i64>()
            .map_err(|e| format!("Invalid integer '{}': {}", s, e))
    }
}

/// Parse a string as u64, treating a "0x" / "0X" prefix as hexadecimal.
fn parse_int_auto_u64(s: &str) -> Result<u64, String> {
    let trimmed = s.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        u64::from_str_radix(&trimmed[2..], 16)
            .map_err(|e| format!("Invalid hex integer '{}': {}", s, e))
    } else {
        trimmed.parse::<u64>()
            .map_err(|e| format!("Invalid integer '{}': {}", s, e))
    }
}

/// Resolve a possibly-negative offset relative to file_size.
/// Negative offsets count backwards from EOF.
fn resolve_offset(offset: i64, file_size: i64) -> i64 {
    if offset < 0 {
        let resolved = file_size + offset;
        if resolved < 0 { 0 } else { resolved }
    } else {
        offset
    }
}

fn main() {
    let matches = Command::new("binrw")
        .version("1.0")
        .about("Binary file read/write CLI tool")
        .subcommand(
            Command::new("read")
                .about("Read bytes from file")
                .arg(Arg::new("filename").required(true).index(1))
                .arg(Arg::new("start").required(true).index(2).allow_hyphen_values(true))
                .arg(Arg::new("end").required(true).index(3).allow_hyphen_values(true)),
        )
        .subcommand(
            Command::new("write")
                .about("Write bytes to file")
                .arg(Arg::new("mode").required(true).index(1))
                .arg(Arg::new("filename").required(true).index(2))
                .arg(Arg::new("offset").required(true).index(3).allow_hyphen_values(true))
                .arg(Arg::new("data").required(true).index(4).allow_hyphen_values(true))
                .arg(Arg::new("extra").index(5))
                .arg(
                    Arg::new("append-zero-past-eof")
                        .long("append-zero-past-eof")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("reverse")
                        .long("reverse")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .action(ArgAction::SetTrue)
                        .help("Treat the data argument as a file path and read its contents as the write data"),
                )
                .arg(
                    Arg::new("file-offset")
                        .long("file-offset")
                        .value_name("OFFSET")
                        .help("Byte offset to start reading from within the data file (requires --file)"),
                ),
        )
        .subcommand(
            Command::new("header")
                .about("Print file header")
                .arg(Arg::new("filename").required(true).index(1)),
        )
        .subcommand(
            Command::new("new")
                .about("Create new file")
                .arg(Arg::new("filename").required(true).index(1)),
        )
        .subcommand(
            Command::new("type")
                .about("Print file type")
                .arg(Arg::new("filename").required(true).index(1)),
        )
        .subcommand(
            Command::new("size")
                .about("Print file size")
                .arg(Arg::new("filename").required(true).index(1)),
        )
        .subcommand(
            Command::new("metadata")
                .about("Print file metadata")
                .arg(Arg::new("filename").required(true).index(1)),
        )
        .subcommand(
            Command::new("copy")
                .about("Copy file")
                .arg(Arg::new("src").required(true).index(1))
                .arg(Arg::new("dest").required(true).index(2)),
        )
        .subcommand(
            Command::new("diff")
                .about("Diff two files")
                .arg(Arg::new("file_a").required(true).index(1))
                .arg(Arg::new("file_b").required(true).index(2)),
        )
        .subcommand(
            Command::new("strings")
                .about("Scan file for printable ASCII strings")
                .arg(Arg::new("filename").required(true).index(1))
                .arg(
                    Arg::new("min-length")
                        .long("min-length")
                        .short('n')
                        .value_name("N")
                        .help("Minimum string length (default: 4)"),
                ),
        )
        .subcommand(
            Command::new("patch")
                .about("Patch raw bytes at an offset; hex bytes may be space-separated or run-together")
                .arg(
                    Arg::new("filename")
                        .required(true)
                        .index(1)
                        .help("Target file to patch"),
                )
                .arg(
                    Arg::new("offset")
                        .required(true)
                        .index(2)
                        .allow_hyphen_values(true)
                        .help("Byte offset (decimal or 0x hex, negative = from end)"),
                )
                .arg(
                    Arg::new("hex")
                        .required(true)
                        .index(3)
                        .num_args(1..)
                        .help("Hex bytes to write, e.g. DE AD BE EF  or  DEADBEEF"),
                ),
        )
        .get_matches();

    // ========== READ ==========
    if let Some(read_matches) = matches.subcommand_matches("read") {
        println!("Read");
        let filename = read_matches.get_one::<String>("filename").unwrap();
        let start_str = read_matches.get_one::<String>("start").unwrap();
        let end_str = read_matches.get_one::<String>("end").unwrap();

        if end_str.trim().eq_ignore_ascii_case("eof") {
            match parse_int_auto_i64(start_str) {
                Ok(offset) => {
                    let size = std::fs::metadata(filename).unwrap().len() as i64;
                    let start_offset = resolve_offset(offset, size);
                    let end_offset = size - 1;
                    let data = read_range_i64_negative_start(filename, start_offset, end_offset as u64);
                    println!("{}", parse_hex_data(data, false).join(" "));
                }
                Err(_) => {
                    eprintln!("Invalid offset: {}", start_str);
                }
            }
        } else {
            let start_i64 = parse_int_auto_i64(start_str);
            let end_i64 = parse_int_auto_i64(end_str);
            match (start_i64, end_i64) {
                (Ok(start), Ok(end)) => {
                    if let Ok(meta) = std::fs::metadata(filename) {
                        let file_size = meta.len() as i64;
                        let start_offset = if start < 0 { file_size + start } else { start };
                        let end_offset = if end < 0 { file_size + end } else { end };
                        if start_offset >= 0 && end_offset >= start_offset {
                            if end_offset >= file_size {
                                let data = read_to_end_i64_negative_offsets(filename, start_offset);
                                println!("{}", parse_hex_data(data, false).join(" "));
                            } else {
                                let data = read_range(filename, start_offset as u64, end_offset as u64);
                                println!("{}", parse_hex_data(data, false).join(" "));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

    // ========== WRITE ==========
    } else if let Some(write_matches) = matches.subcommand_matches("write") {
        println!("Write");
        let mode = write_matches.get_one::<String>("mode").unwrap();
        let filename = write_matches.get_one::<String>("filename").unwrap();
        let offset_arg = write_matches.get_one::<String>("offset").unwrap();
        let data_arg = write_matches.get_one::<String>("data").unwrap();
        let extra_arg = write_matches.get_one::<String>("extra");
        let append_zero_past_eof = write_matches.get_flag("append-zero-past-eof");
        let reverse_flag = write_matches.get_flag("reverse");
        let file_flag = write_matches.get_flag("file");
        let file_offset_arg: Option<u64> = write_matches
            .get_one::<String>("file-offset")
            .map(|s| parse_int_auto_u64(s).expect("--file-offset must be a non-negative integer"));

        let file = File::open(filename).expect("Error opening file for write command");
        let file_size = file.metadata().unwrap().len() as i64;
        drop(file);

        // Helper: resolve data bytes from a string, or read from a file if --file is set
        let resolve_data = |raw: &str| -> Vec<u8> {
            if file_flag {
                let all_bytes = fs::read(raw)
                    .unwrap_or_else(|e| panic!("Failed to read data file '{}': {}", raw, e));
                if let Some(off) = file_offset_arg {
                    let off = off as usize;
                    if off >= all_bytes.len() {
                        Vec::new()
                    } else {
                        all_bytes[off..].to_vec()
                    }
                } else {
                    all_bytes
                }
            } else {
                raw.as_bytes().to_vec()
            }
        };

        let is_range = extra_arg.is_some();

        if mode == "overwrite" {
            if is_range {
                // Range overwrite: offset_arg=first_offset, data_arg=second_offset, extra=actual_data
                let first_i64 = parse_int_auto_i64(offset_arg).unwrap();
                let second_i64 = parse_int_auto_i64(data_arg).unwrap();
                let data = extra_arg.unwrap().clone();

                let first_resolved = resolve_offset(first_i64, file_size);
                let second_resolved = resolve_offset(second_i64, file_size);

                let is_descending = first_resolved > second_resolved;
                let min_idx = first_resolved.min(second_resolved) as usize;
                let max_idx = first_resolved.max(second_resolved) as usize;
                let range_len = max_idx.saturating_sub(min_idx);

                let mut data_bytes = resolve_data(&data);

                // For range overwrite, data length must match range length
                if data_bytes.len() != range_len {
                    return;
                }

                if is_descending || reverse_flag {
                    data_bytes.reverse();
                }

                let mut buffer = fs::read(filename).unwrap_or_default();
                for i in 0..range_len {
                    if min_idx + i < buffer.len() {
                        buffer[min_idx + i] = data_bytes[i];
                    }
                }

                if max_idx > buffer.len() && append_zero_past_eof {
                    buffer.resize(max_idx, 0);
                }

                fs::write(filename, &buffer).unwrap();

            } else if offset_arg == "eof" {
                // Append at EOF
                let mut data_bytes = resolve_data(data_arg);
                if reverse_flag {
                    data_bytes.reverse();
                }
                let mut buffer = fs::read(filename).unwrap_or_default();
                buffer.extend_from_slice(&data_bytes);
                fs::write(filename, &buffer).unwrap();

            } else {
                // Single offset overwrite
                let offset_i64 = parse_int_auto_i64(offset_arg).unwrap_or(0);
                let resolved = resolve_offset(offset_i64, file_size) as usize;

                let mut data_bytes = resolve_data(data_arg);
                if reverse_flag {
                    data_bytes.reverse();
                }
                let data_bytes = &data_bytes;
                let mut buffer = fs::read(filename).unwrap_or_default();

                if resolved < buffer.len() {
                    // In-bounds: overwrite up to EOF
                    let max_write = buffer.len() - resolved;
                    let write_len = data_bytes.len().min(max_write);
                    buffer.splice(
                        resolved..resolved + write_len,
                        data_bytes[..write_len].iter().cloned(),
                    );
                    if append_zero_past_eof && write_len < data_bytes.len() {
                        buffer.extend_from_slice(&data_bytes[write_len..]);
                    }
                    fs::write(filename, &buffer).unwrap();
                } else if resolved == buffer.len() {
                    // Exactly at EOF: append data
                    buffer.extend_from_slice(data_bytes);
                    fs::write(filename, &buffer).unwrap();
                } else if append_zero_past_eof {
                    // Past EOF with flag: pad with zeros, then append data
                    buffer.resize(resolved, 0);
                    buffer.extend_from_slice(data_bytes);
                    fs::write(filename, &buffer).unwrap();
                }
                // else: past EOF without flag, do nothing
            }

        } else if mode == "insert" || mode == "splice" {
            if is_range {
                // Range insert: offset_arg=first_offset, data_arg=second_offset, extra=actual_data
                let first_i64 = parse_int_auto_i64(offset_arg).unwrap();
                let second_i64 = parse_int_auto_i64(data_arg).unwrap();
                let data = extra_arg.unwrap().clone();

                let first_resolved = resolve_offset(first_i64, file_size);
                let second_resolved = resolve_offset(second_i64, file_size);

                let is_descending = first_resolved > second_resolved;
                let min_idx = first_resolved.min(second_resolved) as usize;

                let mut data_bytes = resolve_data(&data);
                if is_descending || reverse_flag {
                    data_bytes.reverse();
                }

                let mut buffer = fs::read(filename).unwrap_or_default();
                if min_idx > buffer.len() {
                    buffer.resize(min_idx, 0);
                }
                buffer.splice(min_idx..min_idx, data_bytes.iter().cloned());
                fs::write(filename, &buffer).unwrap();

            } else if offset_arg == "eof" {
                // Insert/splice at EOF = append
                let mut data_bytes = resolve_data(data_arg);
                if reverse_flag {
                    data_bytes.reverse();
                }
                let mut buffer = fs::read(filename).unwrap_or_default();
                buffer.extend_from_slice(&data_bytes);
                fs::write(filename, &buffer).unwrap();

            } else {
                // Single offset insert/splice
                let offset_i64 = parse_int_auto_i64(offset_arg).unwrap_or(0);
                let resolved = resolve_offset(offset_i64, file_size) as u64;
                let mut data_bytes = resolve_data(data_arg);
                if reverse_flag {
                    data_bytes.reverse();
                }
                write_insert_bytes(filename, resolved, &data_bytes);
            }

        } else {
            panic!("Write command not recognized. Use overwrite, insert, or splice.");
        }

    // ========== HEADER ==========
    } else if let Some(header_matches) = matches.subcommand_matches("header") {
        println!("Header");
        let filename = header_matches.get_one::<String>("filename").unwrap();
        print_header(filename);

    // ========== NEW ==========
    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let filename = new_matches.get_one::<String>("filename").unwrap();
        match File::create(filename.as_str()) {
            Ok(_) => println!("Created new file: {}", filename),
            Err(e) => println!("Failed to create file {}: {}", filename, e),
        }

    // ========== TYPE ==========
    } else if let Some(type_matches) = matches.subcommand_matches("type") {
        println!("Type (Filetype):");
        let filename = type_matches.get_one::<String>("filename").unwrap();
        let file_type = detect_file_type(filename);
        println!("{}", file_type);

    // ========== SIZE ==========
    } else if let Some(size_matches) = matches.subcommand_matches("size") {
        println!("Size");
        let filename = size_matches.get_one::<String>("filename").unwrap();
        let file = File::open(filename.as_str()).unwrap();
        let metadata = file.metadata().unwrap();
        println!("{}", metadata.len());

    // ========== METADATA ==========
    } else if let Some(metadata_matches) = matches.subcommand_matches("metadata") {
        println!("Metadata");
        let filename = metadata_matches.get_one::<String>("filename").unwrap();
        get_file_metadata(filename);

    // ========== COPY ==========
    } else if let Some(copy_matches) = matches.subcommand_matches("copy") {
        let src = copy_matches.get_one::<String>("src").unwrap();
        let dest = copy_matches.get_one::<String>("dest").unwrap();
        match std::fs::copy(src, dest) {
            Ok(_) => println!("Copied {} to {}", src, dest),
            Err(e) => println!("Failed to copy {} to {}: {}", src, dest, e),
        }

    // ========== DIFF ==========
    } else if let Some(diff_matches) = matches.subcommand_matches("diff") {
        let file_a = diff_matches.get_one::<String>("file_a").unwrap();
        let file_b = diff_matches.get_one::<String>("file_b").unwrap();
        println!("TODO: Diff {} {}", file_a, file_b);

    // ========== STRINGS ==========
    } else if let Some(strings_matches) = matches.subcommand_matches("strings") {
        let filename = strings_matches.get_one::<String>("filename").unwrap();
        let min_length: usize = strings_matches
            .get_one::<String>("min-length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(4);
        let results = binrw_cli::scan_strings(filename, min_length);
        if results.is_empty() {
            println!("No strings found.");
        } else {
            for m in results {
                println!("0x{:08X}: {}", m.offset, m.value);
            }
        }

    } else if let Some(patch_matches) = matches.subcommand_matches("patch") {
        // ========== PATCH ==========
        let filename = patch_matches.get_one::<String>("filename").unwrap();
        let offset_str = patch_matches.get_one::<String>("offset").unwrap();
        let hex_args: Vec<&str> = patch_matches
            .get_many::<String>("hex")
            .unwrap()
            .map(|s| s.as_str())
            .collect();

        // Join all hex args and strip non-hex characters (spaces, colons, etc.)
        let raw: String = hex_args.join("");
        let clean: String = raw.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        if clean.len() % 2 != 0 {
            eprintln!("Error: hex bytes must have an even number of hex digits");
            std::process::exit(1);
        }
        if clean.is_empty() {
            eprintln!("Error: no hex bytes provided");
            std::process::exit(1);
        }
        let bytes: Vec<u8> = clean
            .as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect();

        let file_size = std::fs::metadata(filename)
            .expect("Error reading file metadata")
            .len() as i64;
        let offset = match parse_int_auto_i64(offset_str) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
        let start = resolve_offset(offset, file_size) as u64;
        if start >= file_size as u64 {
            eprintln!(
                "Error: offset 0x{:X} is at or past EOF ({})",
                start, file_size
            );
            std::process::exit(1);
        }
        let write_len = bytes.len().min((file_size as u64 - start) as usize);
        use std::io::Seek;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(filename)
            .expect("Error opening file for patching");
        f.seek(std::io::SeekFrom::Start(start))
            .expect("Error seeking in file");
        f.write_all(&bytes[..write_len]).expect("Error writing bytes");
        println!(
            "Patched {} byte(s) at 0x{:X} in {}",
            write_len, start, filename
        );

    } else {
        println!("Command not recognized!");
        std::process::exit(2);
    }
}

// ====================================================================
// Helper functions
// ====================================================================

fn read_id3v1_tag(filename: &str) {
    let data: Vec<u8> = read_to_end_i64_negative_offsets(filename, -128);
    println!("READ TAG DATA: {:?}", data);
}

fn parse_hex_data(data: Vec<u8>, precede_zero_x: bool) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    if precede_zero_x {
        for byte in data {
            output.push(format!("{:#04x}", byte))
        }
    } else {
        for byte in data {
            output.push(format!("{:02x}", byte))
        }
    }
    return output;
}

fn print_header(filename: &str) {
    let parsed_header = parse_header(filename, true);
    println!("{:?}", parsed_header);
}

fn parse_header(filename: &str, precede_zero_x: bool) -> Vec<String> {
    let data: Vec<u8> = fs::read(filename).expect(&format!("Could not open {filename}"));
    let header: Vec<u8> = data[0..4].to_vec();
    let parsed_header = parse_hex_data(header, precede_zero_x);
    return parsed_header;
}

fn read_binary_file_contents(filename: &str) {
    let data: Vec<u8> = fs::read(filename).expect(&format!("Could not open {filename}"));
    let header: [u8; 4] = data[0..4].try_into().unwrap();
    println!("{:?}", header);
}

fn read_to_end(filename: &str, start_byte_inclusive: u64) -> Vec<u8> {
    let file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    return read_range(filename, start_byte_inclusive, size - 1);
}

fn read_to_end_i64_negative_offsets(filename: &str, start_byte_inclusive: i64) -> Vec<u8> {
    let file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len() as i64;
    let start_offset = if start_byte_inclusive < 0 {
        let offset = size + start_byte_inclusive;
        if offset < 0 { 0 } else { offset }
    } else {
        start_byte_inclusive
    };
    if start_offset < 0 || start_offset >= size {
        return vec![];
    }
    read_range(filename, start_offset as u64, (size - 1) as u64)
}

/// Insert bytes into a file at the given offset, shifting existing data to the right.
fn write_insert(filename: &str, start_byte_inclusive: u64, data: String) {
    write_insert_bytes(filename, start_byte_inclusive, data.as_bytes());
}

/// Insert raw bytes into a file at the given offset, shifting existing data to the right.
fn write_insert_bytes(filename: &str, start_byte_inclusive: u64, data_bytes: &[u8]) {
    let mut buffer = fs::read(filename).unwrap_or_default();
    let file_len = buffer.len();
    let start = start_byte_inclusive as usize;

    if data_bytes.is_empty() {
        return;
    }

    if start > file_len {
        // Pad with zeros up to the offset, then append data
        buffer.resize(start, 0);
        buffer.extend_from_slice(data_bytes);
        fs::write(filename, &buffer).unwrap();
    } else {
        // Insert in-place
        buffer.splice(start..start, data_bytes.iter().cloned());
        fs::write(filename, &buffer).unwrap();
    }
}

fn detect_jpg(file_id_info: &FileIDInfo) -> bool {
    let jpeg_start_bytes = vec!["ff".to_owned(), "d8".to_owned()];
    let jpeg_end_bytes = vec!["ff".to_owned(), "d9".to_owned()];
    let first_two_bytes = &file_id_info.first_two_bytes;
    let last_two_bytes = &file_id_info.last_two_bytes;
    return *first_two_bytes == jpeg_start_bytes && *last_two_bytes == jpeg_end_bytes;
}

fn detect_javac(file_id_info: &FileIDInfo) -> bool {
    let java_bytecode_one = "cafebabe";
    let java_bytecode_two = "cafed00d";
    let bytecode_str = file_id_info.first_four_bytes.join("");
    if bytecode_str == java_bytecode_one || bytecode_str == java_bytecode_two {
        return true;
    } else {
        return false;
    }
}

fn get_file_byte_info(filename: &str) -> FileIDInfo {
    let mut file = File::open(filename).unwrap();
    let first_four_bytes = parse_hex_data(read_bytes(&mut file, 0, 4), false);
    let last_two_bytes = parse_hex_data(read_to_end_i64_negative_offsets(filename, -2), false);
    FileIDInfo {
        first_two_bytes: first_four_bytes[0..2].try_into().unwrap(),
        first_four_bytes: first_four_bytes[0..4].try_into().unwrap(),
        last_two_bytes: last_two_bytes[0..2].try_into().unwrap(),
    }
}

#[derive(Debug)]
struct FileByteInfo {
    first_two_bytes: [u8; 2],
    first_four_bytes: [u8; 4],
    last_two_bytes: [u8; 2],
}

#[derive(Debug)]
struct FileIDInfo {
    first_two_bytes: Vec<String>,
    first_four_bytes: Vec<String>,
    last_two_bytes: Vec<String>,
}

fn detect_file_type(filename: &str) -> &str {
    let id_info = get_file_byte_info(filename);
    println!("{:?}", id_info);

    let png_bytes = get_owned_str_vec(vec!["89", "50", "4e", "47"]);
    let gif_bytes = get_owned_str_vec(vec!["47", "49", "46", "38"]);
    let bmp_bytes = get_owned_str_vec(vec!["42", "4d"]);
    let exe_bytes = get_owned_str_vec(vec!["4d", "5a"]);

    if detect_jpg(&id_info) {
        return "jpg";
    } else if id_info.first_four_bytes == png_bytes {
        return "png";
    } else if id_info.first_four_bytes == gif_bytes {
        return "gif";
    } else if id_info.first_two_bytes == bmp_bytes {
        return "bmp";
    } else if id_info.first_two_bytes == exe_bytes {
        return "exe";
    } else if detect_javac(&id_info) {
        return "class";
    } else {
        return "unknown";
    }
}

fn get_owned_str_vec(array: Vec<&str>) -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();
    for element in array {
        vec.push(element.to_owned())
    }
    return vec;
}

fn get_file_metadata(filename: &str) {
    let file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    println!("{}", size);
}
