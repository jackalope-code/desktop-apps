use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Expected usage: binrw read|write|header|type|size|metadata [filename]");
        std::process::exit(1);
    }
    
    let command = &args[1];

    if !(command != "read" && args.len() == 3) && !(command == "read" && args.len() == 5) && !(command == "write" && args.len() == 5) {
        println!("Expected usage: binrw read|write|header|type|size|metadata [filename]");
        println!("{} {}", args.len(), command);
        std::process::exit(1);
    }
    let filename = &args[2];

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
    match command.as_str() {
        "read" | "-r" => {
            println!("Read");
            let aux_arg1 = &args[3];
            let aux_arg2 = &args[4];
            // Try to parse as a u64 and use absolute units when given
            // Try to parse as i64 when negative units are given
            // Either specify a range or byte offset. This should be set with a flag. Settle on a reasonable default.
            // Absolute units and offsets should be able to be mixed in a single command
            // "eof" or "EOf" should work for the second read/write arg to read/write to the end of the file (whether in overwrite or splice write mode)
            if aux_arg2 == "eof" {
                // Parses for offsets
                let data = read_to_end_i64_negative_offsets(filename, aux_arg1.parse::<i64>().unwrap());
                println!("{}", parse_hex_data(data, false).join(" "));
            } else {
                // TODO: Doesn't currently parse for offsets or do smart arg parsing
                let data = read_range(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.parse::<u64>().unwrap());
                // let data = read_bytes(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.parse::<usize>().unwrap());
                println!("{}", parse_hex_data(data, false).join(" "));

            }
        },
        "write" | "-w" => {
            println!("Write");
            let aux_arg1 = &args[3];
            let aux_arg2 = &args[4];
            // write_replace(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.to_string())
            write_insert(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.to_string())
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

fn read_bytes(filename: &str, start_byte_inclusive: u64, num_bytes: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; num_bytes];
    let mut file = File::open(filename).unwrap();
    let seekable = file.seek(SeekFrom::Start(start_byte_inclusive));
    let _ = file.read_exact(&mut buffer);
    return buffer;
}

// Errors reading empty values into buf from read_exact. Buf should be truncated at file size.
fn read_bytes_file(mut file: &File, start_byte_inclusive: u64, num_bytes: usize) -> Vec<u8> {
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    let buffer_size;
    if num_bytes > file_size.try_into().unwrap()  {
        buffer_size = file_size;
    } else {
        buffer_size = num_bytes.try_into().unwrap();
    }
    let mut buffer = vec![0u8; buffer_size.try_into().unwrap()];
    // let mut file = File::open(filename).unwrap();
    let seekable = file.seek(SeekFrom::Start(start_byte_inclusive));
    let _ = file.read_exact(&mut buffer);
    return buffer;
}

// TODO: WIP
fn read_range(filename: &str, start_byte_inclusive: u64, end_byte_inclusive: u64) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    let start_offset = match start_byte_inclusive {
        start_byte_inclusive if start_byte_inclusive >= 0 => start_byte_inclusive,
        _ => start_byte_inclusive+size
    };
    let end_offset = match end_byte_inclusive {
        end_byte_inclusive if end_byte_inclusive >= 0 => end_byte_inclusive,
        _ => end_byte_inclusive+size
    };
    if end_offset <= start_offset  {
        println!("{}", start_offset);
        println!("{}", end_offset);
        panic!("Error in read_range: End byte position cannot be before the start byte position.")
    }
    let buffer_size = end_offset - start_offset;
    return read_bytes(filename, start_offset, buffer_size.try_into().unwrap());
}

fn read_to_end(filename: &str, start_byte_inclusive: u64) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    return read_range(filename, start_byte_inclusive, size-1); // Note size-1. It's size+1 bc I wanted inclusive ranges... I'm off by 1 somewhere???
}

fn read_to_end_i64_negative_offsets(filename: &str, start_byte_inclusive: i64) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    return read_range_i64_negative_start(filename, start_byte_inclusive, size-1);
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
fn read_range_i64_negative_start(filename: &str, start_byte_inclusive: i64, end_byte_inclusive: u64) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    let start_offset: u64 = match start_byte_inclusive {
        start_byte_inclusive if start_byte_inclusive >= 0 => start_byte_inclusive.try_into().unwrap(),
        _ => u64::try_from(isize::try_from(start_byte_inclusive).unwrap()+isize::try_from(size).unwrap()).unwrap()
    };
    let end_offset: u64 = end_byte_inclusive;
    // let end_offset: u64 = match end_byte_inclusive {
    //     end_byte_inclusive if end_byte_inclusive >= 0 => start_byte_inclusive.try_into().unwrap(),
    //     _ => u64::try_from(end_byte_inclusive).unwrap()+size
    // };
    if end_offset <= start_offset  {
        panic!("Error in read_range: End byte position cannot be before the start byte position.")
    }
    let buffer_size = end_offset - start_offset + 1;
    return read_bytes(filename, start_offset.try_into().unwrap(), buffer_size.try_into().unwrap());
}

fn write_replace(filename: &str, start_byte_inclusive: u64, data: String) {
    let mut file = File::open(filename).unwrap();
    let seekable = file.seek(SeekFrom::Start(start_byte_inclusive));
    fs::write(filename, data);
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
        let buf_size: u64 = file_size - start_byte_inclusive; // - (data.len() as u64); // Shouldn't be <0 (I hope)
        println!("BUF SIZE: {}", buf_size);
        let mut buf = vec![0u8; buf_size.try_into().unwrap()];
        let read_size = file.read_to_end(&mut buf).unwrap();
        println!("Read {} bytes starting from {}. File size {}.", read_size, start_byte_inclusive, file_size);
        println!("Buffer size {}", buf.len());
        println!("Seek to {}", start_byte_inclusive);
        file.seek(SeekFrom::Start(start_byte_inclusive.try_into().unwrap()));
        println!("Write data to insert at {} bytes", data.as_bytes().len());
        println!("\"{:?}\"", data.as_bytes());
        file.write(data.as_bytes());
        // if should_append_file_data {
        println!("Write remaining buf: {:?}", &buf);
        file.write(&buf);
    } else {
        file.write(data.as_bytes());
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

fn get_file_byte_info(filename: &str) -> FileIDInfo {
    let first_four_bytes = parse_hex_data(read_bytes(filename, 0, 4), false);
    let last_two_bytes = parse_hex_data(read_to_end_i64_negative_offsets(filename, -2), false);
    return FileIDInfo {
        first_two_bytes: first_four_bytes[0..2].try_into().unwrap(),
        first_four_bytes: first_four_bytes[0..4].try_into().unwrap(),
        last_two_bytes: last_two_bytes[0..2].try_into().unwrap()
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
let java_bytecode_one = "cafebabe";
let java_bytecode_two = "cafed00d";
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