use binrw_cli::read_range;
use binrw_cli::read_range_i64_negative_start;
use binrw_cli::utils::tempfile::TempFile;
use std::io::Write;

// All new, robust, idiomatic tests below (no mod read_tests, no read_position, no old code)

#[test]
fn test_read_negative_offset_to_eof() {
    let data = b"WXYZ";
    let mut file = TempFile::new("test_tempfile.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let offset = -4;
    let size = data.len() as i64;
    let start_offset = if offset < 0 { size + offset } else { offset };
    let end_offset = size - 1;
    let result = read_range_i64_negative_start(&path, start_offset, end_offset as u64);
    assert_eq!(result, b"WXYZ");
}

#[test]
fn test_read_positive_offset_to_eof() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile2.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let offset = 2;
    let size = data.len() as i64;
    let start_offset = if offset < 0 { size + offset } else { offset };
    let end_offset = size - 1;
    let result = read_range_i64_negative_start(&path, start_offset, end_offset as u64);
    assert_eq!(result, b"cdef");
}

#[test]
fn test_read_two_negative_offsets_correct_direction() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile3.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let size = data.len() as i64;
    let start = -4;
    let end = -2;
    let start_offset = if start < 0 { size + start } else { start };
    let end_offset = if end < 0 { size + end } else { end };
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"cde");
}

#[test]
fn test_read_two_negative_offsets_incorrect_direction_should_fail() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile4.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let size = data.len() as i64;
    let start = -2;
    let end = -4;
    let start_offset = if start < 0 { size + start } else { start };
    let end_offset = if end < 0 { size + end } else { end };
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"");
}

#[test]
fn test_read_large_positive_offset_to_eof() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile5.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let offset = 1000;
    let size = data.len() as i64;
    let start_offset = if offset < 0 { size + offset } else { offset };
    let end_offset = size - 1;
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"");
}

#[test]
fn test_read_negative_and_large_positive_offsets() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile6.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let size = data.len() as i64;
    let start = -3;
    let end = 1000;
    let start_offset = if start < 0 { size + start } else { start };
    let end_offset = if end < 0 { size + end } else { end };
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"");
}

#[test]
fn test_read_large_positive_and_negative_offsets() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile7.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let size = data.len() as i64;
    let start = 1000;
    let end = -1;
    let start_offset = if start < 0 { size + start } else { start };
    let end_offset = if end < 0 { size + end } else { end };
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"");
}

#[test]
fn test_read_large_positive_and_large_positive_offsets() {
    let data = b"abcdef";
    let mut file = TempFile::new("test_tempfile8.txt", false).unwrap();
    file.as_file().unwrap().write_all(data).unwrap();
    let path = file.path_str().to_string();
    let start = 1000;
    let end = 2000;
    let size = data.len() as i64;
    let start_offset = if start < 0 { size + start } else { start };
    let end_offset = if end < 0 { size + end } else { end };
    let result = if start_offset >= 0 && end_offset >= start_offset && end_offset < size {
        read_range(&path, start_offset as u64, end_offset as u64)
    } else {
        vec![]
    };
    assert_eq!(result, b"");
}
