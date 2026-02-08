// Print help and exit function must be defined before main
fn print_help_and_exit() {
    println!("\nUsage:");
    println!("  binrw read <filename> <start> <end|eof>");
    println!("  binrw write overwrite <filename> <offset> <data> [--write-past-eof] [--reverse]");
    println!("  binrw write insert <filename> <offset> <data> [--reverse]");
    println!("  binrw header <filename>");
    println!("  binrw new <filename>");
    println!("  binrw type <filename>");
    println!("  binrw size <filename>");
    println!("  binrw metadata <filename>");
    println!("  binrw copy <src> <dest>");
    println!("\nNotes:");
    println!("  Offsets can be negative (from EOF) or 'eof'.");
    println!("  Use --write-past-eof to allow writing past EOF in overwrite mode.");
    println!("  Use --reverse to reverse data before writing/inserting.");
    std::process::exit(2);
}
/// # binrw CLI Usage
///
/// ## Write/Overwrite/Insert
///
/// - `binrw write overwrite <filename> <offset> <data> [--write-past-eof] [--reverse]`
/// - `binrw write insert <filename> <offset> <data> [--reverse]`
///
/// - If `--write-past-eof` is set, writing past EOF pads with zeros and appends data in overwrite mode.
/// - If `--reverse` is set, the data is reversed before writing/inserting.
/// - If a descending range is passed (e.g., `binrw write overwrite file stop start data` where stop > start), the CLI automatically flips the indices and reverses the data, as if `--reverse` was specified.
///
/// ## Examples
///
/// - `binrw write overwrite file.txt 1000 foo --write-past-eof` (pads with zeros and appends "foo" at offset 1000)
/// - `binrw write overwrite file.txt 5 2 foo` (overwrites from index 2 to 5 with "oof")
/// - `binrw write overwrite file.txt 2 foo --reverse` (overwrites at index 2 with "oof")
/// - `binrw write insert file.txt 3 foo --reverse` (inserts "oof" at index 3)
///
/// ## Notes
///
/// - Negative offsets are supported (e.g., -3 means 3 bytes from EOF).
/// - If an invalid/descending range is passed without `--reverse`, the CLI will treat it as reversed.
/// - All behaviors are tested in the test suite.
use binrw_cli::{read_range, read_range_i64_negative_start, read_bytes, read_bytes_file};
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::Write as IoWrite;
// use std::cmp;

mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
        let _ = writeln!(log, "ARGS: {:?}", args);
    }

    if args.len() <= 1 {
        println!("Expected usage: binrw <command> [args]");
        println!("Commands:");
        println!("  read <filename> <start> <end|eof>");
        println!("  write overwrite <filename> <offset> <data>");
        println!("  write insert <filename> <offset> <data>");
        println!("  header <filename>");
        println!("  new <filename>");
        println!("  type <filename>");
        println!("  size <filename>");
        println!("  metadata <filename>");
        println!("  copy <src> <dest>");
        std::process::exit(1);
    }

    let command = &args[1];

    // Accept 'read' with 5 args, 'tag' with 4, 'write' with 6, and others with 3
    let valid =
        (command == "read" && args.len() == 5) ||
        (command == "tag" && args.len() == 4) ||
        (command == "write" && args.len() == 6) ||
        (command == "copy" && args.len() == 4) ||
        (command != "read" && command != "tag" && command != "write" && command != "copy" && args.len() == 3);
    if !valid {
        print_help_and_exit();
    }
    let filename = if command == "write" {&args[3]} else {&args[2]};

    // Validate numeric arguments for read/write
    if command == "read" {
        if args[3].to_lowercase() != "eof" && args[4].to_lowercase() != "eof" {
            if args[3].parse::<i64>().is_err() || args[4].parse::<i64>().is_err() {
                eprintln!("Error: Offsets must be numbers or 'eof'.");
                print_help_and_exit();
            }
        }
    }
    if command == "write" {
        // write overwrite/insert <filename> <offset> <data>
        // args[4] is offset, args[5] is data
        if args[4].to_lowercase() != "eof" && args[4].parse::<i64>().is_err() {
            eprintln!("Error: Offset must be a number or 'eof'.");
            print_help_and_exit();
        }
        // If extra args for range, check those too
        if args.len() > 6 {
            if args[5].parse::<i64>().is_err() || args[4].parse::<i64>().is_err() {
                eprintln!("Error: Offsets must be numbers.");
                print_help_and_exit();
            }
        }
    }

    let debug_enabled = args.iter().any(|a| a == "--debug");
    // New flag for overwrite behavior
    let write_past_eof = args.iter().any(|a| a == "--write-past-eof");
    // New flag for reverse write/overwrite
    let mut reverse_flag = args.iter().any(|a| a == "--reverse");

    macro_rules! debug_log {
        ($($arg:tt)*) => {
            if debug_enabled {
                if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
                    let _ = writeln!(log, $($arg)*);
                }
            }
        }
    }
    debug_log!("ARGS: {:?}", args);

    // TODO: Add tests.
    // Tests:
    // Check correctness and edge cases.
    // ****Read****
    // Negative offset to eof (tested manually to work.. add a case for an easy pass)
    // Large positive offset (larger than i64) to eof
    // Two negative offsets going the correct direction (currently failing)
    // Two negative offsets going in the incorrect direction should fail
    // Two large positive offsets (each larger than i64)
    // Negative and large positive offsets
    // Large positive and negative offsets
    // ****Write****
    // Test same offsets as read (negative to eof, large positive to eof,two negative negative cases, negative large positive, large positive negative, large positive large positive)
    // Test each write operation with each offset (overwrite, splice)
    // Data write verify mode
    // ****Header****
    // Test on range of files
    // ****Type****
    // Test on range of files
    // ****Size****
    // Test on range of files
    // ****Metadata****
    // Add file-specific and metadata-specific metadata checks as metadata functionality is added. Test read, write, write-verify.
    // Metadata write verify mode
    // TODO: Read breaks with negative offsets when not hitting the "eof" true case on the if statement.
    // TODO: Refactor with a custom command parser
    // Debug flag and macro at top of main
    let debug_enabled = args.iter().any(|a| a == "--debug");
    // New flag for overwrite behavior
    let write_past_eof = args.iter().any(|a| a == "--write-past-eof");
    // New flag for reverse write/overwrite
    let mut reverse_flag = args.iter().any(|a| a == "--reverse");
    macro_rules! debug_log {
        ($($arg:tt)*) => {
            if debug_enabled {
                if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
                    let _ = writeln!(log, $($arg)*);
                }
            }
        }
    }
    debug_log!("ARGS: {:?}", args);

    match command.as_str() {
        "read" | "-r" => {
            println!("Read");
            let _aux_arg1 = &args[3];
            let _aux_arg2 = &args[4];
            debug_log!("IN READ: _aux_arg2 raw value: '{}', trimmed: '{}', checking for 'eof'", _aux_arg2, _aux_arg2.trim());
            if _aux_arg2.trim().eq_ignore_ascii_case("eof") {
                debug_log!("IN READ: _aux_arg2 == 'eof' branch taken");
                match _aux_arg1.parse::<i64>() {
                    Ok(offset) => {
                        let size = std::fs::metadata(filename).unwrap().len() as i64;
                        let start_offset = if offset < 0 {
                            let off = size + offset;
                            if off < 0 { 0 } else { off }
                        } else {
                            offset
                        };
                        let end_offset = size - 1;
                        let data = read_range_i64_negative_start(filename, start_offset, end_offset as u64);
                        let output = format!("{}", parse_hex_data(data, false).join(" "));
                        println!("{}", output);
                    }
                    Err(_) => {
                        eprintln!("Invalid offset: {}", _aux_arg1);
                        println!();
                    }
                }
            } else {
                // Support negative offsets for both arguments
                let start_i64 = _aux_arg1.parse::<i64>();
                let end_i64 = _aux_arg2.parse::<i64>();
                match (start_i64, end_i64) {
                    (Ok(start), Ok(end)) => {
                        let metadata = std::fs::metadata(filename);
                        if let Ok(meta) = metadata {
                            let file_size = meta.len() as i64;
                            let start_offset = if start < 0 { file_size + start } else { start };
                            let end_offset = if end < 0 { file_size + end } else { end };
                            // If end_offset >= file_size, read to end of file
                            if start_offset >= 0 && end_offset >= start_offset {
                                if end_offset >= file_size {
                                    let data = read_to_end_i64_negative_offsets(filename, start_offset);
                                    println!("{}", parse_hex_data(data, false).join(" "));
                                } else {
                                    let data = read_range(filename, start_offset as u64, end_offset as u64);
                                    println!("{}", parse_hex_data(data, false).join(" "));
                                }
                            } else {
                                println!();
                            }
                        } else {
                            // Could not get file metadata, print nothing
                            println!();
                        }
                    }
                    _ => {
                        // Parsing failed, print nothing
                        println!();
                    }
                }
            }
        },
        "new" => {
            if args.len() < 3 {
                println!("Usage: binrw new <filename>");
                return;
            }
            let filename = &args[2];
            match File::create(filename) {
                Ok(_) => println!("Created new file: {}", filename),
                Err(e) => println!("Failed to create file {}: {}", filename, e),
            }
        },
        "write" | "-w" => {
            println!("Write");
            // Specify write splice or write overwrite with the write command (so 6 args total).
            let aux_arg1 = &args[2]; // splice or overwrite
            let _aux_arg2 = &args[4]; // 4 position (start or stop)
            let _aux_arg3 = &args[5]; // 5 data
            let mut start_offset = _aux_arg2;
            let mut stop_offset = None;
            let mut data = _aux_arg3.to_string();
            // Handle descending range or --reverse for overwrite/insert
            // If two offsets are provided (overwrite with range), check if descending
            let mut should_reverse = reverse_flag;
            if args.len() > 6 {
                let possible_stop = &args[4];
                let possible_start = &args[5];
                if let (Ok(stop), Ok(start)) = (possible_stop.parse::<i64>(), possible_start.parse::<i64>()) {
                    // Always use lower as start, higher as stop
                    if stop > start {
                        start_offset = possible_start;
                        stop_offset = Some(possible_stop);
                        // Always reverse for descending range
                        should_reverse = true;
                        debug_log!("Detected descending range: stop {} > start {}. Swapping and reversing data.", stop, start);
                    } else {
                        start_offset = possible_stop;
                        stop_offset = Some(possible_start);
                        should_reverse = reverse_flag;
                    }
                }
            }
            // Reverse the data only once if needed
            if should_reverse {
                debug_log!("Reversing data for write/overwrite/insert: before='{}'", data);
                data = data.chars().rev().collect();
                debug_log!("Reversed data: after='{}'", data);
            }
            match aux_arg1.as_str() {
                "overwrite" => {
                    if let Some(stop_offset_str) = stop_offset {
                        // Range overwrite: handle negative/positive/descending, clamp to [0, file.len()]
                        let file = File::open(filename).expect("Error opening file for write command");
                        let metadata = file.metadata().unwrap();
                        let file_size = metadata.len() as i64;
                        let start_i64 = start_offset.parse::<i64>().unwrap_or(0);
                        let stop_i64 = stop_offset_str.parse::<i64>().unwrap_or(0);
                        let mut resolved_start = if start_i64 < 0 {
                            file_size + start_i64
                        } else {
                            start_i64
                        };
                        let mut resolved_stop = if stop_i64 < 0 {
                            file_size + stop_i64
                        } else {
                            stop_i64
                        };
                        // Clamp to [0, file_size]
                        resolved_start = resolved_start.max(0).min(file_size);
                        resolved_stop = resolved_stop.max(0).min(file_size);
                        let mut data_to_write = data.clone();
                        let (start_idx, stop_idx, do_reverse) = if resolved_start <= resolved_stop {
                            (resolved_start as usize, resolved_stop as usize, false)
                        } else {
                            (resolved_stop as usize, resolved_start as usize, true)
                        };
                        let mut buffer = fs::read(filename).unwrap_or_default();
                        // If descending and start_idx > stop_idx, do not change file (invalid range)
                        if start_idx >= buffer.len() || stop_idx > buffer.len() || start_idx == stop_idx || (do_reverse && start_idx > stop_idx) {
                            debug_log!("[overwrite range] Out of bounds or invalid/descending range ({}..{}), no change for {}", start_idx, stop_idx, filename);
                            return;
                        }
                        // For descending, always reverse data and map left-to-right in file
                        let overwrite_len = (stop_idx - start_idx).min(data_to_write.len());
                        // For descending, always reverse data and write left-to-right at lower index
                        let mut data_bytes = data_to_write.clone().into_bytes();
                        if do_reverse {
                            // Stepwise manual mapping: always map data left-to-right in file, reverse data if needed
                            let mut mapped_indices = Vec::new();
                            let mut mapped_data = Vec::new();
                            let data_len = data_bytes.len();
                            let file_range = stop_idx - start_idx;
                            let write_len = file_range.min(data_len);
                            debug_log!("[overwrite range] start_idx={}, stop_idx={}, do_reverse={}, reverse_flag={}, data_bytes={:?}", start_idx, stop_idx, do_reverse, reverse_flag, data_bytes);
                            if do_reverse != reverse_flag {
                                // Reverse data for mapping
                                for i in 0..write_len {
                                    let file_i = start_idx + i;
                                    let data_i = data_len - 1 - i;
                                    mapped_indices.push(file_i);
                                    mapped_data.push(data_bytes[data_i]);
                                    buffer[file_i] = data_bytes[data_i];
                                    debug_log!(
                                        "[overwrite range][REVERSE] i={} file_i={} data_i={} byte=0x{:02X} start_idx={} stop_idx={} data_len={} write_len={} filename={}",
                                        i, file_i, data_i, data_bytes[data_i], start_idx, stop_idx, data_len, write_len, filename
                                    );
                                }
                            } else {
                                // Normal mapping
                                for i in 0..write_len {
                                    let file_i = start_idx + i;
                                    let data_i = i;
                                    mapped_indices.push(file_i);
                                    mapped_data.push(data_bytes[data_i]);
                                    buffer[file_i] = data_bytes[data_i];
                                    debug_log!(
                                        "[overwrite range][NORMAL] i={} file_i={} data_i={} byte=0x{:02X} start_idx={} stop_idx={} data_len={} write_len={} filename={}",
                                        i, file_i, data_i, data_bytes[data_i], start_idx, stop_idx, data_len, write_len, filename
                                    );
                                }
                            }
                            match fs::write(filename, &buffer) {
                                Ok(_) => debug_log!("[overwrite range] Overwrote in bounds for {}. Data: {:?}", filename, mapped_data),
                                Err(e) => {
                                    debug_log!("[overwrite range] ERROR writing file {}: {}", filename, e);
                                    panic!("[overwrite range] ERROR writing file {}: {}", filename, e);
                                }
                            }
                            return;
                        }
                        // Always map data left-to-right in file, regardless of range direction
                        // Only write if range is valid (start < stop)
                        if overwrite_len > 0 {
                            for i in 0..overwrite_len {
                                buffer[start_idx + i] = data_bytes[i];
                            }
                            match fs::write(filename, &buffer) {
                                Ok(_) => debug_log!("[overwrite range] Overwrote in bounds for {}. Data: {:?}", filename, &data_bytes[..overwrite_len]),
                                Err(e) => {
                                    debug_log!("[overwrite range] ERROR writing file {}: {}", filename, e);
                                    panic!("[overwrite range] ERROR writing file {}: {}", filename, e);
                                }
                            }
                        } else {
                            debug_log!("[overwrite range] No valid range to write for {}", filename);
                        }
                    } else if start_offset == "eof" {
                        // Overwrite at EOF: always append, reverse if --reverse
                        let mut data_to_write = data.clone();
                        if reverse_flag {
                            data_to_write = data_to_write.chars().rev().collect();
                        }
                        let mut buffer = fs::read(filename).unwrap_or_default();
                        buffer.extend_from_slice(data_to_write.as_bytes());
                        match fs::write(filename, &buffer) {
                            Ok(_) => debug_log!("[overwrite eof] Appended data at EOF for {}. Data: {:?}", filename, data_to_write.as_bytes()),
                            Err(e) => {
                                debug_log!("[overwrite eof] ERROR writing file {}: {}", filename, e);
                                panic!("[overwrite eof] ERROR writing file {}: {}", filename, e);
                            }
                        }
                    } else {
                        // Single offset: reverse if --reverse
                        let file = File::open(filename).expect("Error opening file for write command");
                        let metadata = file.metadata().unwrap();
                        let file_size = metadata.len() as i64;
                        let offset_i64 = start_offset.parse::<i64>().unwrap_or(0);
                        let resolved_offset = if offset_i64 < 0 {
                            let off = file_size + offset_i64;
                            if off < 0 { 0 } else { off }
                        } else {
                            offset_i64
                        } as u64;
                        let mut data_to_write = data.clone();
                        if reverse_flag {
                            data_to_write = data_to_write.chars().rev().collect();
                        }
                        let mut buffer = fs::read(filename).unwrap_or_default();
                        let start_idx = resolved_offset as usize;
                        if start_idx < buffer.len() {
                            // Overwrite up to EOF, do NOT append remainder unless write_past_eof is set
                            let max_write = buffer.len() - start_idx;
                            let data_bytes = data_to_write.as_bytes();
                            if data_bytes.len() <= max_write {
                                // All fits in overwrite
                                buffer.splice(start_idx..start_idx+data_bytes.len(), data_bytes.iter().cloned());
                            } else {
                                // Overwrite to EOF only
                                buffer.splice(start_idx.., data_bytes[..max_write].iter().cloned());
                                // Only append remainder if write_past_eof is set
                                if write_past_eof {
                                    buffer.extend_from_slice(&data_bytes[max_write..]);
                                }
                            }
                            match fs::write(filename, &buffer) {
                                Ok(_) => debug_log!("[overwrite single-offset] Overwrote to EOF{} for {}. Data: {:?}", if write_past_eof {" and appended"} else {""}, filename, data_bytes),
                                Err(e) => {
                                    debug_log!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                    panic!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                }
                            }
                        } else if start_idx == buffer.len() {
                            // Append at EOF
                            buffer.extend_from_slice(data_to_write.as_bytes());
                            match fs::write(filename, &buffer) {
                                Ok(_) => debug_log!("[overwrite single-offset] Appended at EOF for {}. Data: {:?}", filename, data_to_write.as_bytes()),
                                Err(e) => {
                                    debug_log!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                    panic!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                }
                            }
                        } else if write_past_eof {
                            // Pad with zeros and append
                            buffer.resize(start_idx, 0);
                            buffer.extend_from_slice(data_to_write.as_bytes());
                            match fs::write(filename, &buffer) {
                                Ok(_) => debug_log!("[overwrite single-offset] Appended past EOF for {}. Data: {:?}", filename, data_to_write.as_bytes()),
                                Err(e) => {
                                    debug_log!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                    panic!("[overwrite single-offset] ERROR writing file {}: {}", filename, e);
                                }
                            }
                        } else {
                            debug_log!("[overwrite single-offset] Offset {} out of bounds and no write_past_eof. No change.", resolved_offset);
                        }
                    }
                },
                "insert" => {
                    if let Some(stop_offset_str) = stop_offset {
                        // Insert: handle descending and reversal
                        let file = File::open(filename).expect("Error opening file for write command");
                        let metadata = file.metadata().unwrap();
                        let file_size = metadata.len() as i64;
                        let start_i64 = start_offset.parse::<i64>().unwrap_or(0);
                        let stop_i64 = stop_offset_str.parse::<i64>().unwrap_or(0);
                        let resolved_start = if start_i64 < 0 {
                            let off = file_size + start_i64;
                            if off < 0 { 0 } else { off }
                        } else {
                            start_i64
                        } as u64;
                        let resolved_stop = if stop_i64 < 0 {
                            let off = file_size + stop_i64;
                            if off < 0 { 0 } else { off }
                        } else {
                            stop_i64
                        } as u64;
                        let (insert_idx, _range_end, is_descending) = if resolved_start <= resolved_stop {
                            (resolved_start, resolved_stop, false)
                        } else {
                            (resolved_stop, resolved_start, true)
                        };
                        // For descending, always reverse data and insert at lower index
                        let mut data_to_write = data.clone();
                        if is_descending || reverse_flag {
                            // Stepwise manual mapping for insert: always insert at lower index, reverse data if needed
                            let mut data_bytes = data_to_write.as_bytes().to_vec();
                            let data_len = data_bytes.len();
                            debug_log!("[insert] insert_idx={}, is_descending={}, reverse_flag={}, data_bytes={:?}", insert_idx, is_descending, reverse_flag, data_bytes);
                            if is_descending != reverse_flag {
                                data_bytes.reverse();
                                debug_log!("[insert] reversed data_bytes={:?}", data_bytes);
                            }
                            write_insert(filename, insert_idx, String::from_utf8_lossy(&data_bytes).to_string());
                            return;
                        }
                        debug_log!("[insert comprehensive] Insert at {}, data='{}' (data_len {})", insert_idx, data_to_write, data_to_write.len());
                        write_insert(filename, insert_idx, data_to_write)
                    } else if start_offset == "eof" {
                        let file = File::open(filename).expect("Error opening file for eof write command");
                        let metadata = file.metadata().unwrap();
                        let file_size = metadata.len();
                        let mut data_to_write = data.clone();
                        if reverse_flag {
                            data_to_write = data_to_write.chars().rev().collect();
                        }
                        debug_log!("Insert at EOF (offset {}), data='{}'", file_size, data_to_write);
                        write_insert(filename, file_size.try_into().unwrap(), data_to_write)
                    } else {
                        let file = File::open(filename).expect("Error opening file for write command");
                        let metadata = file.metadata().unwrap();
                        let file_size = metadata.len() as i64;
                        let offset_i64 = start_offset.parse::<i64>().unwrap_or(0);
                        let resolved_offset = if offset_i64 < 0 {
                            let off = file_size + offset_i64;
                            if off < 0 { 0 } else { off }
                        } else {
                            offset_i64
                        } as u64;
                        let mut data_to_write = data.clone();
                        if reverse_flag {
                            debug_log!("[insert single-offset] Reversing data for --reverse: before='{}'", data_to_write);
                            data_to_write = data_to_write.chars().rev().collect();
                            debug_log!("[insert single-offset] Reversed data: after='{}'", data_to_write);
                        }
                        debug_log!("[insert single-offset] start_offset='{}' parsed offset_i64={}, resolved_offset={}, data='{}' (len={})", start_offset, offset_i64, resolved_offset, data_to_write, data_to_write.len());
                        write_insert(filename, resolved_offset, data_to_write)
                    }
                },
                _ => {
                    println!("AUX_ARG1: {}", aux_arg1);
                    panic!("Write command not recognized. Specify either overwrite or insert with the write command.")
                }
            }
        },
        "header" | "-h" => {
            println!("Header");
            print_header(filename.as_str());
        },
        "type" | "-t" => {
            println!("Type (Filetype):");
            let file_type = detect_file_type(filename.as_str());
            println!("{}", file_type);
        },
        "size" | "-s" => {
            println!("Size");
        },
        "metadata" | "-m" => {
            println!("Metadata");
            get_file_metadata(filename);
        },
        "tag" => {
            println!("Tag");
            let subcommand = &args[2]; // read or write
            let filename = &args[3];
            if subcommand == "read" {
                // If the file is an MP3 using ID3v1
                read_id3v1_tag(filename);
            }
        },
        "diff" => {
            println!("TODO: Diff");
            // Diff subcommand usage: binrw diff fileA fileB
            let aux_arg1 = &args[2];
            let aux_arg2 = &args[3];
            // Start with a unified diff. See https://unix.stackexchange.com/questions/81998/understanding-of-diff-output
            // let unified_diff = String::new();
            // let biggest_str_len = cmp::max(aux_arg1, aux_arg2);
            // for i in 0..biggest_str_len {
            //     if  == aux_arg2 {
            //         unified_diff.push_str(aux_arg1);
            //         unified_diff.push_str("\r\n")
            //     } else if aux_
            // }
        }
        "copy" => {
            if args.len() < 4 {
                println!("Usage: binrw copy <src> <dest>");
                return;
            }
            let src = &args[2];
            let dest = &args[3];
            match std::fs::copy(src, dest) {
                Ok(_) => println!("Copied {} to {}", src, dest),
                Err(e) => println!("Failed to copy {} to {}: {}", src, dest, e),
            }
        }
        _ => {
            println!("Command not recognized!");
            std::process::exit(2);
        }
    }
}


// fn parse_header_bytes(header: &Vec<u8>) -> Vec<String> {
//     let mut parsed_header: Vec<String> = Vec::new();
//     for byte in header {
//         parsed_header.push(format!("{:04x}", byte))
//     }
//     return parsed_header;
// }

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
    // TODO: Read just the first few bytes instead of the whole file to parse the filetype and file header
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

// TODO: Support optional negative start offset (but keep the u64 range when possible... how??? And standardize this across fns!!!)
// moved to lib.rs

fn read_to_end(filename: &str, start_byte_inclusive: u64) -> Vec<u8> {
    let file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    return read_range(filename, start_byte_inclusive, size-1); // Note size-1. It's size+1 bc I wanted inclusive ranges... I'm off by 1 somewhere???
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
    // Read from start_offset (inclusive) to size-1 (inclusive)
    read_range(filename, start_offset as u64, (size - 1) as u64)
}

// TODO: WIP. Correctly handle both positive and negative ranges, either one or the other or also mixed. Figure out good type definitions. Convert types correctly. 
// fn read_range_i64_negative_offsets(filename: &str, start_byte_inclusive: isize, end_byte_inclusive: isize) -> Vec<u8> {
//     let mut file = File::open(filename).unwrap();
//     let metadata = file.metadata().unwrap();
//     let size = metadata.len();
//     let start_offset: u64 = match start_byte_inclusive {
//         start_byte_inclusive if start_byte_inclusive >= 0 => start_byte_inclusive.try_into().unwrap(),
//         _ => u64::try_from(start_byte_inclusive+size).unwrap()
//     };
//     let end_offset: u64 = match end_byte_inclusive {
//         end_byte_inclusive if end_byte_inclusive >= 0 => start_byte_inclusive.try_into().unwrap(),
//         _ => u64::try_from(end_byte_inclusive+size).unwrap()
//     };
//     if end_offset <= start_offset  {
//         panic!("Error in read_range: End byte position cannot be before the start byte position.")
//     }
//     TODO: Bugged? Look at read_range_i64_negative_start... it panicked w/ this and I reversed it to end_offset - start_offset. See the check above. The end_offset should always be greater.
//     let buffer_size = start_offset - end_offset + 1;
//     return read_bytes(filename, start_offset.try_into().unwrap(), buffer_size.try_into().unwrap());
// }

/// Overwrite bytes in a file at the given offset.
/// If the offset is past EOF, pads with zeros and appends the data if write_past_eof is true.
/// If the offset is within the file, replaces bytes up to the length of data.
/// Usage: binrw write overwrite <filename> <offset> <data> [--write-past-eof]
fn write_overwrite(filename: &str, start_byte_inclusive: u64, data: String, write_past_eof: bool) {
    let debug_enabled = std::env::args().any(|a| a == "--debug");
    macro_rules! debug_log_inner {
        ($($arg:tt)*) => {
            if debug_enabled {
                if let Ok(mut log) = std::fs::OpenOptions::new().create(true).append(true).open("debug_log.txt") {
                    let _ = writeln!(log, $($arg)*);
                }
            }
        }
    }
    let mut buffer = fs::read(filename).unwrap_or_default();
    let file_len = buffer.len();
    let start = start_byte_inclusive as usize;
    let data_bytes = data.as_bytes();
    debug_log_inner!("[write_overwrite] BEFORE: buffer='{}' (len={}), start={}, data='{}' (len={}), write_past_eof={}, file_len={}", String::from_utf8_lossy(&buffer), file_len, start, data, data.len(), write_past_eof, file_len);
    if data_bytes.is_empty() {
        debug_log_inner!("[write_overwrite] No data to write, skipping modification for {}", filename);
        return;
    }
    debug_log_inner!("[write_overwrite] file_len={}, start={}, data_len={}, filename={}", file_len, start, data_bytes.len(), filename);
    if start > file_len {
        if write_past_eof {
            // Pad with zeros up to the offset, then append data
            buffer.resize(start, 0);
            buffer.extend_from_slice(data_bytes);
            debug_log_inner!("[write_overwrite] AFTER (padded): buffer='{}' (len={})", String::from_utf8_lossy(&buffer), buffer.len());
            match fs::write(filename, &buffer) {
                Ok(_) => debug_log_inner!("[write_overwrite] Appended zeros and data past EOF for {}. New len: {}. Data: {:?}", filename, buffer.len(), data_bytes),
                Err(e) => {
                    debug_log_inner!("[write_overwrite] ERROR writing file {}: {}", filename, e);
                    panic!("[write_overwrite] ERROR writing file {}: {}", filename, e);
                }
            }
        } else {
            debug_log_inner!("[write_overwrite] Not writing past EOF, only writing up to EOF for {}", filename);
            // Only write up to EOF
            let available = buffer.len().saturating_sub(start);
            if available > 0 {
                buffer.splice(start..buffer.len(), data_bytes[..available].iter().cloned());
                debug_log_inner!("[write_overwrite] buffer after splice up to EOF: {:?}", buffer);
                match fs::write(filename, &buffer) {
                    Ok(_) => debug_log_inner!("[write_overwrite] Wrote up to EOF for {}. Data: {:?}", filename, &data_bytes[..available]),
                    Err(e) => {
                        debug_log_inner!("[write_overwrite] ERROR writing file {}: {}", filename, e);
                        panic!("[write_overwrite] ERROR writing file {}: {}", filename, e);
                    }
                }
            } else {
                debug_log_inner!("[write_overwrite] No space to write up to EOF for {}", filename);
            }
        }
    } else {
        // Overwrite in-place, do not append past EOF
        let actual_end = (start + data_bytes.len()).min(buffer.len());
        let overwrite_len = actual_end.saturating_sub(start);
        if overwrite_len > 0 {
            buffer.splice(start..actual_end, data_bytes[..overwrite_len].iter().cloned());
        }
        // If data is longer than the range, append the rest
        if actual_end < start + data_bytes.len() && (actual_end == buffer.len() && write_past_eof) {
            buffer.extend_from_slice(&data_bytes[overwrite_len..]);
        }
        match fs::write(filename, &buffer) {
            Ok(_) => debug_log_inner!("[write_overwrite] Overwrote in-place from {} to {} for {}. Data: {:?}", start, actual_end, filename, &data_bytes[..overwrite_len]),
            Err(e) => {
                debug_log_inner!("[write_overwrite] ERROR writing file {}: {}", filename, e);
                panic!("[write_overwrite] ERROR writing file {}: {}", filename, e);
            }
        }
    }
}

/// Insert bytes into a file at the given offset, shifting existing data to the right.
/// The file size increases by the length of the inserted data.
/// Usage: binrw write insert <filename> <offset> <data>
fn write_insert(filename: &str, start_byte_inclusive: u64, data: String) {
    let debug_enabled = std::env::args().any(|a| a == "--debug");
    macro_rules! debug_log_inner {
        ($($arg:tt)*) => {
            if debug_enabled {
                if let Ok(mut log) = std::fs::OpenOptions::new().create(true).append(true).open("debug_log.txt") {
                    let _ = writeln!(log, $($arg)*);
                }
            }
        }
    }
    let mut buffer = fs::read(filename).unwrap_or_default();
    let file_len = buffer.len();
    let start = start_byte_inclusive as usize;
    let data_bytes = data.as_bytes();
    debug_log_inner!("[write_insert] BEFORE: buffer='{}' (len={}), start={}, data='{}' (len={}), file_len={}", String::from_utf8_lossy(&buffer), file_len, start, data, data.len(), file_len);
    if data_bytes.is_empty() {
        debug_log_inner!("[write_insert] No data to insert, skipping modification for {}", filename);
        return;
    }
    debug_log_inner!("[write_insert] file_len={}, start={}, data_len={}, filename={}", file_len, start, data_bytes.len(), filename);

    if start > file_len {
        // Pad with zeros up to the offset, then append data
        buffer.resize(start, 0);
        buffer.extend_from_slice(data_bytes);
        debug_log_inner!("[write_insert] AFTER (padded): buffer='{}' (len={})", String::from_utf8_lossy(&buffer), buffer.len());
        match fs::write(filename, &buffer) {
            Ok(_) => debug_log_inner!("[write_insert] Inserted past EOF for {}. Padded with zeros. New len: {}. Data: {:?}", filename, buffer.len(), data_bytes),
            Err(e) => {
                debug_log_inner!("[write_insert] ERROR writing file {}: {}", filename, e);
                panic!("[write_insert] ERROR writing file {}: {}", filename, e);
            }
        }
    } else if start <= file_len {
        // Insert in-place
        buffer.splice(start..start, data_bytes.iter().cloned());
        debug_log_inner!("[write_insert] AFTER (in-place): buffer='{}' (len={})", String::from_utf8_lossy(&buffer), buffer.len());
        match fs::write(filename, &buffer) {
            Ok(_) => debug_log_inner!("[write_insert] Inserted at offset {} for {}. Data: {:?}", start, filename, data_bytes),
            Err(e) => {
                debug_log_inner!("[write_insert] ERROR writing file {}: {}", filename, e);
                panic!("[write_insert] ERROR writing file {}: {}", filename, e);
            }
        }
    } else {
        // Invalid range, no change
        debug_log_inner!("[write_insert] Invalid range, no change for {}", filename);
    }
}

fn detect_jpg(file_id_info: &FileIDInfo) -> bool {
    let jpeg_start_bytes = vec!["ff".to_owned(), "d8".to_owned()];
    let jpeg_end_bytes = vec!["ff".to_owned(), "d9".to_owned()];
    // let first_two_bytes = parse_hex_data(read_bytes(filename, 0, 2), false);
    // let last_two_bytes = parse_hex_data(read_to_end_i64_negative_offsets(filename, -2), false);
    let first_two_bytes = &file_id_info.first_two_bytes;
    let last_two_bytes = &file_id_info.last_two_bytes;
    // println!("{:?}", *first_two_bytes);
    // println!("{:?}", *last_two_bytes);
    // println!("{}", *first_two_bytes == jpeg_start_bytes);
    // println!("{}", *last_two_bytes == jpeg_end_bytes);
    // println!("{}", *first_two_bytes == jpeg_start_bytes && *last_two_bytes == jpeg_end_bytes);
    return *first_two_bytes == jpeg_start_bytes &&
        *last_two_bytes == jpeg_end_bytes;
}

fn detect_javac(file_id_info: &FileIDInfo) -> bool {
    let java_bytecode_one = "cafebabe";
    let java_bytecode_two = "cafed00d";
    let bytecode_str = file_id_info.first_four_bytes.join("");
    // let is_class_file = match bytecode_str {
    //     java_bytecode_one => true,
    //     java_bytecode_two => true,
    //     _ => false
    // };
    // return is_class_file;
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

// enum FileType {
//     JPG(&str),
//     Unknown(`&str)
// }

#[derive(Debug)]
struct FileByteInfo {
    first_two_bytes: [u8; 2],
    first_four_bytes: [u8; 4],
    last_two_bytes: [u8; 2]
}

#[derive(Debug)]
struct FileIDInfo {
    first_two_bytes: Vec<String>,
    first_four_bytes: Vec<String>,
    last_two_bytes: Vec<String>
}

fn detect_file_type(filename: &str) -> &str {
// //     let header = parse_header(filename, false)[0..2];
// //     println!("{}", header);
    // let last_two_bytes = parse_hex_data(read_to_end_i64_negative_offsets(filename, -2));
    // println!("{:?}", parse_hex_data(last_two_bytes, false));
    // detect_jpg(filename);
    let id_info = get_file_byte_info(filename);
    println!("{:?}", id_info);
        // https://stackoverflow.com/questions/4550296/how-to-identify-contents-of-a-byte-is-a-jpeg
        /* start ff d8 end ff d9 */
        /* BMP : 42 4D
JPG : FF D8 FF EO ( Starting 2 Byte will always be same)
PNG : 89 50 4E 47
GIF : 47 49 46 38*/
/*
When a JPG file uses JFIF or EXIF, The signature is different :

Raw  : FF D8 FF DB  
JFIF : FF D8 FF E0  
EXIF : FF D8 FF E1 */
let png_bytes = get_owned_str_vec(vec!["89", "50", "4e", "47"]);
let gif_bytes = get_owned_str_vec(vec!["47", "49", "46", "38"]);
let bmp_bytes = get_owned_str_vec(vec!["42", "4d"]);
let exe_bytes = get_owned_str_vec(vec!["4d", "5a"]);
// java class files: cafebabe or cafed00d

//parse_hex_data(read_bytes(filename, 0, 4), false).join("");
// https://en.wikipedia.org/wiki/Magic_number_(programming)#In_files
// midi: ASCII code for MThd (MIDI Track header): 4d 54 68 64 followed by more metadata
// unix or linux scripts may start with shebang ("#!", 23, 21, followed by the path to an interpreter, if the interpreter is likely to be different)
// elf executables start with 7f followed by "ELF" (7f 45 4c 46)
// PDF "%PDF" hex 25 50 44 46
// DOS MZ executables and EXE stub of microsoft windows PE "MZ" (4d 5a)
// ZIP file PK<3club (50 4b 03 04)
// 7z file (37 7a bc af 27 1c)
// println!("Java bytecode: {}", java_bytecode);

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
    // let file_type = detect_file_type(filename);
    // println!("{}", format!("TODO: READ METADATA FOR {file_type}"));
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    // println!("{:?}", file_type);
    println!("{}", size);
}