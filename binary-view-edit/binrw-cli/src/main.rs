use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Expected usage: binrw read|write|header|type|size|metadata [filename]");
        std::process::exit(1);
    }
    
    let command = &args[1];

    if !(command != "read" && args.len() == 3) && !(command == "read" && args.len() == 5) {
        println!("Expected usage: binrw read|write|header|type|size|metadata [filename]");
        println!("{}", args.len());
        std::process::exit(1);
    }
    let filename = &args[2];

    // TODO: Add tests.
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
        end_byte_inclusive if end_byte_inclusive >= 0 => start_byte_inclusive,
        _ => end_byte_inclusive+size
    };
    if end_offset <= start_offset  {
        panic!("Error in read_range: End byte position cannot be before the start byte position.")
    }
    let buffer_size = start_offset - end_offset + 1;
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

fn write_replace(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn write_append(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn detect_file_type(filename: &str) -> &str {
// //     let header = parse_header(filename, false)[0..2];
// //     println!("{}", header);
//     let last_two_bytes = read_range_i64_negative_start(filename, -2)
//     println!("{}", last_two_bytes);
//     let file_type = match header {
//         // https://stackoverflow.com/questions/4550296/how-to-identify-contents-of-a-byte-is-a-jpeg
//         vec!["ff", "d8"] => "jpg" /* start ff d8 end ff d9 */,
//         /* BMP : 42 4D
// JPG : FF D8 FF EO ( Starting 2 Byte will always be same)
// PNG : 89 50 4E 47
// GIF : 47 49 46 38*/
// /*
// When a JPG file uses JFIF or EXIF, The signature is different :

// Raw  : FF D8 FF DB  
// JFIF : FF D8 FF E0  
// EXIF : FF D8 FF E1 */
//         _ => "unknown"
//     };
//     return file_type;
    return "unknown";
}

fn get_file_metadata(filename: &str) {
    let mut file = File::open(filename).unwrap();
    // let file_type = detect_file_type(filename);
    // println!("{}", format!("TODO: READ METADATA FOR {file_type}"));
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    // println!("{:?}", file_type);
    println!("{}", size);
}
