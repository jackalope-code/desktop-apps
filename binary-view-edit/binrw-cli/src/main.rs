use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Expected usage: binrw read|write [filename]");
        std::process::exit(1);
    }
    
    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "read" | "-r" => {
            println!("Read ");
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
        }
        "metadata" | "-m" => {
        }
        _ => {
            println!("Command not recognized!");
            std::process::exit(2);
        }
    }
    println!("Hello, world!");
}


// fn parse_header_bytes(header: &Vec<u8>) -> Vec<String> {
//     let mut parsed_header: Vec<String> = Vec::new();
//     for byte in header {
//         parsed_header.push(format!("{:04x}", byte))
//     }
//     return parsed_header;
// }

fn parse_hex_data(data: Vec<u8>, precede_zero_x: bool) -> String {
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
    return output.join(" ");
}

fn print_header(filename: &str) {
    let parsed_header = parse_header(filename, true);
    println!("{}", parsed_header);
}

fn parse_header(filename: &str, precede_zero_x: bool) -> String {
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

fn seek_read(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn write_replace(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn write_append(start_byte_inclusive: i32, end_byte_inclusive: i32) {

}

fn detect_file_type(filename: &str) -> &str {
    let header = parse_header(filename, false);
    println!("{}", header);
    let file_type = match header.as_str() {
        // https://stackoverflow.com/questions/4550296/how-to-identify-contents-of-a-byte-is-a-jpeg
        "ff d8 ff e0" => "jpg" /* start ff d8 end ff d9 */,
        /* BMP : 42 4D
JPG : FF D8 FF EO ( Starting 2 Byte will always be same)
PNG : 89 50 4E 47
GIF : 47 49 46 38*/
/*
When a JPG file uses JFIF or EXIF, The signature is different :

Raw  : FF D8 FF DB  
JFIF : FF D8 FF E0  
EXIF : FF D8 FF E1 */
        _ => "unknown"
    };
    return file_type;
}

fn get_file_metadata(filename: &str) {
    let file_type = detect_file_type(filename);
    println!("{}", format!("TODO: READ METADATA FOR {file_type}"));
}
