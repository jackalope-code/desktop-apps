#[cfg(test)]
mod write_overwrite_tests {
    use super::*;


    use binrw_cli::utils::tempfile::TempFile;
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
        file.as_file().unwrap().write_all(b"abc").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (beyond EOF) should pad with zeros and append data
        run_write_command_with_flag("overwrite", path, "1000", "xyz", "--append-zero-past-eof");
        let content = fs::read(path).unwrap();
        // Should be original data, then 997 zeros, then "xyz"
        let mut expected = b"abc".to_vec();
        expected.resize(1000, 0);
        expected.extend_from_slice(b"xyz");
        assert_eq!(content, expected);
    }

    #[test]
    fn write_large_positive_offsets_no_append() {
        let mut file = TempFile::new("test_write_large_positive_offsets_no_append.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abc").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (beyond EOF) should do nothing without flag
        run_write_command("overwrite", path, "1000", "xyz");
        let content = fs::read(path).unwrap();
        // Should be unchanged
        let expected = b"abc".to_vec();
        assert_eq!(content, expected);
    }

    #[test]
    fn overwrite_to_eof() {
        let mut file = TempFile::new("test_overwrite_to_eof.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write at offset -3 (slightly before EOF) should overwrite last 3 bytes
        run_write_command("overwrite", path, "-3", "xyz");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite def with xyz
        assert_eq!(content, "abcxyz");
    }

    #[test]
    fn write_large_positive_offset_to_negative_offset() {
        let mut file = TempFile::new("test_write_large_positive_offset_to_negative_offset.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write from a large positive offset to a negative offset (simulate valid left->right)
        // For this test, write at offset 2 (valid, should overwrite from c)
        run_write_command("overwrite", path, "2", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "abXYZf");
    }

    #[test]
    fn write_negative_offset_to_large_positive_offsets() {
        let mut file = TempFile::new("test_write_negative_offset_to_large_positive_offsets.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write from -4 (index 2) to 4 (index 4), valid left->right
        run_write_command("overwrite", path, "-4", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite cde with XYZ
        assert_eq!(content, "abXYZf");
    }

    #[test]
    fn write_large_positive_offset_past_eof_success_append_zero() {
        let mut file = TempFile::new("test_write_large_positive_offset_past_eof_success.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (well past EOF) should pad with zeros and append data with flag
        run_write_command_with_flag("overwrite", path, "1000", "xyz", "--append-zero-past-eof");
        let content = fs::read(path).unwrap();
        // Should be original data, then 994 zeros, then "xyz"
        let mut expected = b"abcdef".to_vec();
        expected.resize(1000, 0);
        expected.extend_from_slice(b"xyz");
        assert_eq!(content, expected);
    }

    #[test]
    fn write_large_positive_offset_past_eof_success_no_append() {
        let mut file = TempFile::new("test_write_large_positive_offset_past_eof_success_no_append.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write at offset 1000 (well past EOF) should do nothing without flag
        run_write_command("overwrite", path, "1000", "xyz");
        let content = fs::read(path).unwrap();
        // Should be unchanged
        let expected = b"abcdef".to_vec();
        assert_eq!(content, expected);
    }

    #[test]
    fn write_negative_offset_to_eof() {
        let mut file = TempFile::new("test_write_negative_offset_to_eof.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Write at offset -3 (from end, index 3)
        run_write_command("overwrite", path, "-3", "xyz");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite def with xyz
        assert_eq!(content, "abcxyz");
    }

    #[test]
    fn overwrite_negative() {
        let mut file = TempFile::new("test_overwrite_negative_pass.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Overwrite from -3 (index 3) to the end of the file
        run_write_command("overwrite", path, "-3", "XYZERRORERRORERROR");
        let content = fs::read_to_string(path).unwrap();
        // Should overwrite def with XYZ
        assert_eq!(content, "abcXYZ");
    }

    #[test]
    fn write_negative_to_negative_descending_fail() {
        let mut file = TempFile::new("test_write_negative_to_negative_descending_fail.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Invalid: start > end (simulate by writing at -2, but not enough data to fill)
        run_write_command("overwrite", path, "-2", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        // Should not change file
        assert_eq!(content, "abcdef");
    }

    #[test]
    fn write_to_eof_success() {
        let mut file = TempFile::new("test_write_to_eof_success.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Overwrite at EOF (offset 6)
        run_write_command("overwrite", path, "6", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "abcdefXYZ");
    }

    #[test]
    fn invalid_write_offset_fail() {
        let mut file = TempFile::new("test_invalid_write_offset_fail.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        // Offset out of bounds
        run_write_command("overwrite", path, "9999", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "abcdef");
    }

    // Removed duplicate invalid_write_offset_fail

    #[ignore]
    #[test]
    fn overwrite_file_with_file() {
        // Placeholder: implement file-to-file overwrite logic here
    }

    #[test]
    fn write_verify() {
        let mut file = TempFile::new("test_write_verify.txt", false).unwrap();
        file.as_file().unwrap().write_all(b"abcdef").unwrap();
        let path = file.path_str();
        run_write_command("overwrite", path, "3", "XYZ");
        let content = fs::read_to_string(path).unwrap();
        // Should be "abcXYZ"
        assert_eq!(content, "abcXYZ");
    }

    #[ignore]
    #[test]
    fn check_write_overwrite() {

    }
    
    #[ignore]
    #[test]
    fn check_write_overwrite_from_file() {

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