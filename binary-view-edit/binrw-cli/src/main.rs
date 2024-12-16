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
    let data: Vec<u8> = fs::read(filename).expect(&format!("Could not open {filename}"));
    let header: Vec<u8> = data[0..4].to_vec();
    let parsed_header = parse_hex_data(header, false);
    println!("{}", parsed_header);
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
