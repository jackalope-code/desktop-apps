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

    match command.as_str() {
        "read" | "-r" => {
            println!("Read");
            let aux_arg1 = &args[3];
            let aux_arg2 = &args[4];
            let data = read_range(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.parse::<u64>().unwrap());
            // let data = read_range(filename, aux_arg1.parse::<u64>().unwrap(), aux_arg2.parse::<usize>().unwrap());
            println!("{}", parse_hex_data(data, false).join(" "));
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
            println!("{}", detect_file_type(filename.as_str()));
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
    let metadata = file.metadata().unwrap();
    let size = metadata.len();
    let start_offset = match start_byte_inclusive {
        (start_byte_inclusive >= 0) => start_byte_inclusive,
        _ => start_byte_inclusive+size
    }
    let end_offset = match end_byte_inclusive {
        end_byte_inclusive >= 0 => end_byte_inclusive,
        _ => end_byte_inclusive+size
    }
    if end_offset <= start_offset  {
        panic!("Error in read_range: End byte position cannot be before the start byte position.")
    }
    let buffer_size = start_offset - end_offset + 1;
    return read_bytes(filename, start_offset, buffer_size.try_into().unwrap());
}

fn write_replace(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn write_append(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn detect_file_type(filename: &str) -> &str {
//     let header = parse_header(filename, false)[0..2];
//     println!("{}", header);
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
    println!("{:?}", file_type);
    println!("{}", size);
}
