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
    // Debug log CLI arguments
    use std::fs::OpenOptions;
    use std::io::Write as IoWrite;
    let args: Vec<String> = env::args().collect();
    if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
        let _ = writeln!(log, "ARGS: {:?}", args);
    }

    if args.len() <= 1 {
        println!("Expected usage: binrw read|write|header|new|type|size|metadata [filename]");
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
        println!("Expected usage: binrw read|write|header|type|size|metadata [filename]");
        println!("{} {}", args.len(), command);
        std::process::exit(1);
    }
    let filename = if command == "write" {&args[3]} else {&args[2]};

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
        }
        "write" | "-w" => {
            println!("Write");
            // Specify write splice or write overwrite with the write command (so 6 args total).
            // TODO: Translates and breaks down eof parsing here and in read. Move into other functions!!!
            // 0 (program)
            // 1 command
            // 2 subcommand (splice or overwrite)
            let aux_arg1 = &args[2]; // splice or overwrite
            // 3 filename
            let _aux_arg2 = &args[4]; // 4 position
            let _aux_arg3 = &args[5]; // 5 data (allow arg to read in file data instead)
            // write_replace(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.to_string())
            let write_command = match aux_arg1.as_str() {
                "overwrite" => write_replace,
                "splice" => write_insert,
                _ => {
                    println!("AUX_ARG1: {}", aux_arg1);
                    panic!("Write command not recognized. Specify either overwrite or splice with the write command.")
                }
            };
            if _aux_arg2 == "eof" {
                let file = File::open(filename).expect("Error opening file for eof write command");
                let metadata = file.metadata().unwrap();
                let file_size = metadata.len();
                write_command(filename, file_size.try_into().unwrap(), _aux_arg3.to_string())
            } else {
                // Support negative offsets for overwrite/splice
                let file = File::open(filename).expect("Error opening file for write command");
                let metadata = file.metadata().unwrap();
                let file_size = metadata.len() as i64;
                let offset_i64 = _aux_arg2.parse::<i64>().unwrap_or(0);
                let resolved_offset = if offset_i64 < 0 {
                    let off = file_size + offset_i64;
                    if off < 0 { 0 } else { off }
                } else {
                    offset_i64
                } as u64;
                write_command(filename, resolved_offset, _aux_arg3.to_string())
            }
            // write_insert(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.to_string())
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

// TODO: WIP
// moved to lib.rs

fn write_replace(filename: &str, start_byte_inclusive: u64, data: String) {
    // Read the file into a buffer
    let mut buffer = fs::read(filename).unwrap_or_default();
    let file_len = buffer.len();
    let mut start = start_byte_inclusive as isize;
    if start < 0 {
        start = file_len as isize + start;
    }
    if start < 0 {
        start = 0;
    }
    let start = start as usize;
    let data_bytes = data.as_bytes();
    // If data is empty, do nothing
    if data_bytes.is_empty() {
        return;
    }
    let end = start + data_bytes.len();
    // Descending/invalid overwrites are blocked below
    if end <= start {
        // Descending/invalid overwrite
        return;
    }
    if start > buffer.len() {
        // Do not write if offset is past EOF (only allow at EOF)
        return;
    } else if start == buffer.len() {
        // Only allow appending at EOF
        buffer.extend_from_slice(data_bytes);
    } else {
        // Overwrite up to EOF, append remainder if any
        let overwrite_end = end.min(buffer.len());
        buffer.splice(start..overwrite_end, data_bytes[..(overwrite_end-start)].iter().cloned());
        if end > buffer.len() {
            buffer.extend_from_slice(&data_bytes[(buffer.len()-start).max(0)..]);
        }
    }
    let _ = fs::write(filename, buffer);
}

fn write_insert(filename: &str, start_byte_inclusive: u64, data: String) {
    let mut file = File::options().read(true).write(true).open(filename).unwrap();
    // let seekable = file.seek(SeekFrom::Start(start_byte_inclusive+1));
    println!("Opening file for write_insert");
    // println!("Planning to read {} bytes, starting from {}", data.len(), start_byte_inclusive);
    // let buf = read_bytes_file(&file, start_byte_inclusive, data.len())
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    // let should_append_file_data = file_size == 0;

    if file_size > 0 {
        // TODO: Some issue where buf_size is doubled or loading when it should be 0. So just skip conditionally on buf_size
        let buf_size: u64 = file_size - start_byte_inclusive; // - (data.len() as u64); // Shouldn't be <0 (I hope)
        if buf_size > 0 {
            println!("BUF SIZE: {}", buf_size);
            let mut buf = vec![0u8; buf_size.try_into().unwrap()];
            let read_size = file.read_to_end(&mut buf).unwrap();
            println!("Read {} bytes starting from {}. File size {}.", read_size, start_byte_inclusive, file_size);
            println!("Buffer size {}", buf.len()); // TODO: Why is this double BUF SIZE?
            println!("Seek to {}", start_byte_inclusive);
            let _ = file.seek(SeekFrom::Start(start_byte_inclusive.try_into().unwrap()));
            println!("Write data to insert at {} bytes", data.as_bytes().len());
            println!("\"{:?}\"", data.as_bytes());
            let _ = file.write(data.as_bytes());
            println!("Write remaining buf: {:?}", &buf);
            let _ = file.write(&buf[buf.len()-read_size..]); // TODO: WHY DO I NEED THIS HACK?!?!
        } else {
            let _ = file.seek(SeekFrom::Start(start_byte_inclusive.try_into().unwrap()));
            let _ = file.write(data.as_bytes());
        }
    } else {
        let _ = file.write(data.as_bytes());
    }
    // }
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

    let file_type: &str = match id_info {
        id_info if detect_jpg(&id_info) => {return "jpg"},
        id_info if id_info.first_four_bytes == png_bytes => {return "png"},
        id_info if id_info.first_four_bytes == gif_bytes => {return "gif"},
        id_info if id_info.first_two_bytes == bmp_bytes => {return "bmp"},
        id_info if id_info.first_two_bytes == exe_bytes => {return "exe"}
        id_info if detect_javac(&id_info) => {return "class"}
        _ => "unknown",
    };
    return file_type;
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