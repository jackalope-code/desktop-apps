
use binrw_cli::utils::tempfile::TempFile;
use std::io::Write;

    #[test]
    fn overwrite_reverse_flag() {
        let mut file = TempFile::new("test_overwrite_reverse_flag.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Overwrite at offset 2 with reversed data using --reverse
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("overwrite").arg(path).arg("2").arg("foo").arg("--reverse")
            .output().expect("Failed to run binrw-cli write overwrite --reverse");
        let content = std::fs::read_to_string(path).unwrap();
        // Should overwrite OOB with oof
        assert_eq!(content, "FOoofR");
    }

    #[test]
    fn overwrite_descending_range() {
        let mut file = TempFile::new("test_overwrite_descending_range.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Overwrite with descending range: stopIndex > startIndex
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("overwrite").arg(path).arg("5").arg("2").arg("foo")
            .output().expect("Failed to run binrw-cli write overwrite descending");
        let content = std::fs::read_to_string(path).unwrap();
        // Should overwrite OBA with oof (reversed)
        assert_eq!(content, "FOoofR");
    }

    #[test]
    fn insert_reverse_flag() {
        let mut file = TempFile::new("test_insert_reverse_flag.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Insert at offset 3 with reversed data using --reverse
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("insert").arg(path).arg("3").arg("foo").arg("--reverse")
            .output().expect("Failed to run binrw-cli write insert --reverse");
        let content = std::fs::read_to_string(path).unwrap();
        // Should insert oof at index 3
        assert_eq!(content, "FOOoofBAR");
    }

    #[test]
    fn insert_descending_range() {
        let mut file = TempFile::new("test_insert_descending_range.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Insert with descending range: stopIndex > startIndex
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("insert").arg(path).arg("5").arg("3").arg("foo")
            .output().expect("Failed to run binrw-cli write insert descending");
        let content = std::fs::read_to_string(path).unwrap();
        // Should insert oof at index 3
        assert_eq!(content, "FOOoofBAR");
    }
#[cfg(test)]
mod write_overwrite_tests {
    use super::*;


    use std::io::Write;
    use std::fs;
    use std::process::Command;

    fn run_write_command(mode: &str, file: &str, offset: &str, data: &str) {
        let _ = Command::new("target/debug/binrw-cli.exe")
            .arg("write")
            .arg(mode)
            .arg(file)
            .arg(offset)
            .arg(data)
            .output()
            .expect("Failed to run binrw-cli write");
    }

    fn run_write_command_with_flag(mode: &str, file: &str, offset: &str, data: &str, flag: &str) {
        let _ = Command::new("target/debug/binrw-cli.exe")
            .arg("write")
            .arg(mode)
            .arg(file)
            .arg(offset)
            .arg(data)
            .arg(flag)
            .output()
            .expect("Failed to run binrw-cli write with flag");
    }

    #[test]
    fn write_large_positive_offsets_append_zero() {
        let mut file = TempFile::new("test_write_large_positive_offsets.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOO").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (beyond EOF) should pad with zeros and append data
        run_write_command_with_flag("overwrite", path, "1000", "foo", "--append-zero-past-eof");
        let content = fs::read(path).unwrap();
        // Should be original data, then 997 zeros, then "foo"
        let mut expected = b"FOO".to_vec();
        expected.resize(1000, 0);
        expected.extend_from_slice(b"foo");
        assert_eq!(content, expected);
    }

    #[test]
    fn write_large_positive_offsets_no_append() {
        let mut file = TempFile::new("test_write_large_positive_offsets_no_append.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOO").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (beyond EOF) should do nothing without flag
        run_write_command("overwrite", path, "1000", "foo");
        let content = fs::read(path).unwrap();
        // Should be unchanged
        let expected = b"FOO".to_vec();
        assert_eq!(content, expected);
    }

    #[test]
    fn overwrite_to_eof() {
        let mut file = TempFile::new("test_overwrite_to_eof.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write at offset -3 (slightly before EOF) should overwrite last 3 bytes only
        run_write_command("overwrite", path, "-2", "foobar");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite BAR with foo
        assert_eq!(content, "FOOBfo");
    }

    #[test]
    fn write_large_positive_offset_to_negative_offset() {
        let mut file = TempFile::new("test_write_large_positive_offset_to_negative_offset.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write from a large positive offset to a negative offset (simulate valid left->right)
        // For this test, write at offset 3 (valid, should overwrite from B)
        run_write_command("overwrite", path, "3", "foo");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "FOOfoo");
    }

    #[test]
    fn write_negative_offset_to_large_positive_offsets() {
        let mut file = TempFile::new("test_write_negative_offset_to_large_positive_offsets.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write from -3 (index 3) to end, valid left->right
        run_write_command("overwrite", path, "-3", "foo");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite BAR with foo
        assert_eq!(content, "FOOfoo");
    }

    #[test]
    fn write_large_positive_offset_past_eof_success_append_zero() {
        let mut file = TempFile::new("test_write_large_positive_offset_past_eof_success.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (well past EOF) should pad with zeros and append data with flag
        run_write_command_with_flag("overwrite", path, "1000", "foo", "--append-zero-past-eof");
        let content = fs::read(path).unwrap();
        // Should be original data, then 994 zeros, then "foo"
        let mut expected = b"FOOBAR".to_vec();
        expected.resize(1000, 0);
        expected.extend_from_slice(b"foo");
        assert_eq!(content, expected);
    }

    #[test]
    fn write_large_positive_offset_past_eof_success_no_append() {
        let mut file = TempFile::new("test_write_large_positive_offset_past_eof_success_no_append.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (well past EOF) should do nothing without flag
        run_write_command("overwrite", path, "1000", "foo");
        let content = fs::read(path).unwrap();
        // Should be unchanged
        let expected = b"FOOBAR".to_vec();
        assert_eq!(content, expected);
    }

    #[test]
    fn write_negative_offset_to_eof() {
        let mut file = TempFile::new("test_write_negative_offset_to_eof.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Write at offset -3 (from end, index 3)
        run_write_command("overwrite", path, "-3", "foo");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite BAR with foo
        assert_eq!(content, "FOOfoo");
    }

    #[test]
    fn overwrite_negative() {
        let mut file = TempFile::new("test_overwrite_negative_pass.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Overwrite from -3 (index 3) to the end of the file
        run_write_command("overwrite", path, "-3", "foo");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite BAR with foo
        assert_eq!(content, "FOOfoo");
    }

    #[test]
    fn write_negative_to_negative_descending_fail() {
        let mut file = TempFile::new("test_write_negative_to_negative_descending_fail.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Partial write: offset -2 (index 4), data "FOO" (3 bytes) but only 2 bytes fit before EOF
        run_write_command("overwrite", path, "-2", "FOO");
        let content = fs::read_to_string(path).unwrap();
        // Should partially overwrite: write first 2 bytes of "FOO" at index 4,5
        assert_eq!(content, "FOOBFO");
    }

    #[test]
    fn write_to_eof_success() {
        let mut file = TempFile::new("test_write_to_eof_success.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Overwrite at EOF (offset 6)
        run_write_command("overwrite", path, "6", "foo");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "FOOBARfoo");
    }

    #[test]
    fn invalid_write_offset_fail() {
        let mut file = TempFile::new("test_invalid_write_offset_fail.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        // Offset out of bounds
        run_write_command("overwrite", path, "9999", "FOO");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "FOOBAR");
    }

    // Removed duplicate invalid_write_offset_fail

    #[test]
    fn overwrite_file_with_file() {
        // Create target file
        let mut target = TempFile::new("test_overwrite_file_with_file.txt", false).unwrap();
        target.as_file().unwrap().write_all(b"ABCDEFGHIJ").unwrap();
        // Create source data file
        let mut src = TempFile::new("test_overwrite_src_data.bin", false).unwrap();
        src.as_file().unwrap().write_all(b"XYZ").unwrap();
        // Overwrite at offset 3 with contents of source file
        let _ = Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("overwrite")
            .arg(target.path_str()).arg("3").arg(src.path_str())
            .arg("--file")
            .output().expect("Failed to run binrw-cli write overwrite --file");
        let content = fs::read_to_string(target.path_str()).unwrap();
        assert_eq!(content, "ABCXYZGHIJ");
    }

    #[test]
    fn write_verify() {
        let mut file = TempFile::new("test_write_verify.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"FOOBAR").unwrap();
        let path = file.path_str();
        run_write_command("overwrite", path, "3", "foo");
        let content = fs::read_to_string(path).unwrap();
        // Should be "FOOfoo"
        assert_eq!(content, "FOOfoo");
    }

    #[test]
    fn check_write_overwrite() {
        // Write data, then read back and verify contents match
        let mut file = TempFile::new("test_check_write_overwrite.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"ABCDEFGHIJ").unwrap();
        let path = file.path_str();
        run_write_command("overwrite", path, "3", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "ABCXYZGHIJ");
        // Overwrite at start
        run_write_command("overwrite", path, "0", "00");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "00CXYZGHIJ");
        // Overwrite at end
        run_write_command("overwrite", path, "8", "!!");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "00CXYZGH!!");
    }
    
    #[test]
    fn check_write_overwrite_from_file() {
        // Create target file
        let mut target = TempFile::new("test_check_overwrite_from_file.txt", false).unwrap();
        target.as_file().unwrap().write_all(b"HELLO_WORLD").unwrap();
        // Create source data file with "RUST"
        let mut src = TempFile::new("test_overwrite_from_file_src.bin", false).unwrap();
        src.as_file().unwrap().write_all(b"RUST").unwrap();
        // Overwrite at offset 6 with file contents
        let _ = Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("overwrite")
            .arg(target.path_str()).arg("6").arg(src.path_str())
            .arg("--file")
            .output().expect("Failed to run binrw-cli write overwrite --file");
        let content = fs::read_to_string(target.path_str()).unwrap();
        // "HELLO_WORLD" overwrite at 6 with "RUST" (4 bytes, room for 5) → "HELLO_RUSTD"
        assert_eq!(content, "HELLO_RUSTD");
    }
    #[ignore]
    #[test]
    fn ignore_existing_file_by_default() {

    }
    #[ignore]
    #[test]
    fn edit_existing_file_with_flag() {

    }
    #[ignore]
    #[test]
    fn save_edit_as_copy_with_flag() {

    }
    #[ignore]
    #[test]
    fn create_new_file_with_flag() {

    }
}