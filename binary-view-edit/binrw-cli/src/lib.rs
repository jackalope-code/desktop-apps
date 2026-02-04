pub fn read_bytes(file: &mut File, start_byte_inclusive: u64, num_bytes: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; num_bytes];
    let _seekable = file.seek(SeekFrom::Start(start_byte_inclusive));
    let _ = file.read_exact(&mut buffer);
    buffer
}

pub fn read_bytes_file(file: &mut File, start_byte_inclusive: u64, num_bytes: usize) -> Vec<u8> {
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    let buffer_size = if num_bytes > file_size.try_into().unwrap() {
        file_size as usize
    } else {
        num_bytes
    };
    read_bytes(file, start_byte_inclusive, buffer_size)
}

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

pub fn read_range(filename: &str, start_byte_inclusive: u64, end_byte_inclusive: u64) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let _size = metadata.len();
    let start_offset = start_byte_inclusive;
    let end_offset = end_byte_inclusive;
    if end_offset < start_offset  {
        println!("{}", start_offset);
        println!("{}", end_offset);
        panic!("Error in read_range: End byte position cannot be before the start byte position.")
    }
    let buffer_size = end_offset - start_offset + 1;
    read_bytes_file(&mut file, start_offset, buffer_size.try_into().unwrap())
}

pub fn read_range_i64_negative_start(filename: &str, start_byte_inclusive: i64, end_byte_inclusive: u64) -> Vec<u8> {
    if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
        let _ = writeln!(log, "called read_range_i64_negative_start: filename={} start_byte_inclusive={} end_byte_inclusive={}", filename, start_byte_inclusive, end_byte_inclusive);
    }
    if let Ok(mut log) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
        let _ = writeln!(log, "start_offset={} end_offset={} size={} buffer_size={}",
            if start_byte_inclusive < 0 {
                let size = std::fs::metadata(filename).unwrap().len() as i64;
                let offset = size + start_byte_inclusive;
                if offset < 0 { 0 } else { offset }
            } else {
                start_byte_inclusive
            },
            end_byte_inclusive as i64,
            std::fs::metadata(filename).unwrap().len() as i64,
            (end_byte_inclusive as i64 - if start_byte_inclusive < 0 {
                let size = std::fs::metadata(filename).unwrap().len() as i64;
                let offset = size + start_byte_inclusive;
                if offset < 0 { 0 } else { offset }
            } else {
                start_byte_inclusive
            } + 1) as usize
        );
    }
    println!("[DEBUG] read_range_i64_negative_start: filename={} start_byte_inclusive={} end_byte_inclusive={}", filename, start_byte_inclusive, end_byte_inclusive);
    let mut file = File::open(filename).unwrap();
    let metadata = file.metadata().unwrap();
    let size = metadata.len() as i64;
    let start_offset: i64 = if start_byte_inclusive < 0 {
        let offset = size + start_byte_inclusive;
        if offset < 0 { 0 } else { offset }
    } else {
        start_byte_inclusive
    };
    let end_offset = end_byte_inclusive as i64;
    if end_offset < start_offset || start_offset < 0 || end_offset > size - 1 {
        return vec![];
    }
    let buffer_size = (end_offset - start_offset + 1) as usize;
    read_bytes(&mut file, start_offset as u64, buffer_size)
}
// pub mod funcs;

// TODO: Modules? Importing? Exporting?

pub mod utils;

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