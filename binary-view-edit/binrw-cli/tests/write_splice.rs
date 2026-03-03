use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io;
use std::io::Write;

// mod utils;
use binrw_cli::utils::tempfile::TempFile;

#[cfg(test)]
mod write_splice_tests {
    use super::*;
    
    fn write_splice_data(path_str: &str, position: &str, data: &str, print_output: bool) -> String {
        //  binrw write splice file_path position hello
        let output = Command::new("./target/debug/binrw-cli.exe")
            .arg("write")
            .arg("splice")
            .arg(path_str)
            .arg(position)
            .arg(data)
            .output()
            .expect("Failed to execute binrw-cli.");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if print_output {
            println!("STDOUT: {}", stdout);
            println!("STDERR: {}", stderr);
        }
        return stdout.to_string();
    }

    fn write_hello_once(path_str: &str, position: &str, print_output: bool) -> String {
        return write_splice_data(path_str, position, "hello", print_output);
    }

    #[test]
    fn write_large_positive_offsets() {
        // Splice at offset far past EOF — should pad with zeros then insert
        let mut temp_file = TempFile::new("splice_large_pos.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABC").unwrap();
        }
        write_splice_data(temp_file.path_str(), "100", "XY", false);
        let data = fs::read(temp_file.path_str()).unwrap();
        // Should be: ABC + 97 zeros + XY = 102 bytes
        assert_eq!(data.len(), 102);
        assert_eq!(&data[0..3], b"ABC");
        assert_eq!(&data[100..102], b"XY");
        assert!(data[3..100].iter().all(|&b| b == 0));
    }
    
    #[test]
    fn write_large_positive_offset_to_negative_offset() {
        // Splice at a positive offset within bounds
        let mut temp_file = TempFile::new("splice_pos_to_neg.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"FOOBAR").unwrap();
        }
        write_splice_data(temp_file.path_str(), "3", "XY", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        // Insert XY at index 3: FOO + XY + BAR
        assert_eq!(data, "FOOXYBAR");
    }

    #[test]
    fn write_negative_offset_to_large_positive_offsets() {
        // Splice at a negative offset (from end)
        let mut temp_file = TempFile::new("splice_neg_to_pos.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"FOOBAR").unwrap();
        }
        write_splice_data(temp_file.path_str(), "-3", "XY", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        // Insert XY at index 3 (6-3=3): FOO + XY + BAR
        assert_eq!(data, "FOOXYBAR");
    }

    #[test]
    fn write_large_positive_offset_to_eof() {
        // Splice past EOF — same as large positive offsets case
        let mut temp_file = TempFile::new("splice_pos_to_eof.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"AB").unwrap();
        }
        write_splice_data(temp_file.path_str(), "50", "CD", false);
        let data = fs::read(temp_file.path_str()).unwrap();
        assert_eq!(data.len(), 52);
        assert_eq!(&data[0..2], b"AB");
        assert_eq!(&data[50..52], b"CD");
    }

    #[test]
    fn write_negative_offset_to_eof() {
        // Splice at negative offset -1 (last position)
        let mut temp_file = TempFile::new("splice_neg_to_eof.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABCDEF").unwrap();
        }
        write_splice_data(temp_file.path_str(), "-1", "XY", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        // Insert XY at index 5 (6-1=5): ABCDE + XY + F
        assert_eq!(data, "ABCDEXYF");
    }
    
    #[test]
    fn write_negative_to_negative_ascending_pass() {
        // Splice at a negative offset within bounds (ascending)
        let mut temp_file = TempFile::new("splice_neg_neg_asc.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABCDEF").unwrap();
        }
        write_splice_data(temp_file.path_str(), "-4", "XY", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        // Insert XY at index 2 (6-4=2): AB + XY + CDEF
        assert_eq!(data, "ABXYCDEF");
    }

    #[test]
    fn write_negative_to_negative_descending_fail() {
        // Splice at -2 (index 4) inserts before last 2 chars
        let mut temp_file = TempFile::new("splice_neg_neg_desc.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABCDEF").unwrap();
        }
        write_splice_data(temp_file.path_str(), "-2", "XY", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        // Insert XY at index 4 (6-2=4): ABCD + XY + EF
        assert_eq!(data, "ABCDXYEF");
    }

    #[test]
    fn write_to_eof_success() {
        // Splice at "eof" should append
        let mut temp_file = TempFile::new("splice_to_eof.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABC").unwrap();
        }
        write_splice_data(temp_file.path_str(), "eof", "DEF", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        assert_eq!(data, "ABCDEF");
    }

    #[test]
    fn invalid_write_offset_fail() {
        // Splice at a huge offset past EOF — pads with zeros
        let mut temp_file = TempFile::new("splice_invalid_offset.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABC").unwrap();
        }
        write_splice_data(temp_file.path_str(), "9999", "X", false);
        let data = fs::read(temp_file.path_str()).unwrap();
        // Should pad to offset 9999, then insert X
        assert_eq!(data.len(), 10000);
        assert_eq!(&data[0..3], b"ABC");
        assert_eq!(data[9999], b'X');
    }

    #[test]
    fn write_verify() {
        // Splice then verify by reading back
        let mut temp_file = TempFile::new("splice_verify.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"FOOBAR").unwrap();
        }
        write_splice_data(temp_file.path_str(), "3", "!!!", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        assert_eq!(data, "FOO!!!BAR");
    }
    
    
    #[test]
    fn check_write_splice() {
        // Basic splice verification: insert at beginning, middle, end
        let mut temp_file = TempFile::new("splice_check.test.txt", false).expect("Error creating temp file");
        if let Some(f) = temp_file.as_file() {
            use std::io::Write;
            f.write_all(b"ABCD").unwrap();
        }
        // Insert at beginning
        write_splice_data(temp_file.path_str(), "0", "Z", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        assert_eq!(data, "ZABCD");
        // Insert at end (offset = length)
        write_splice_data(temp_file.path_str(), "5", "Q", false);
        let data = fs::read_to_string(temp_file.path_str()).unwrap();
        assert_eq!(data, "ZABCDQ");
    }
    
    #[test]
    fn check_write_splice_from_file() {
        // Create target file
        let mut target = TempFile::new("test_splice_from_file.txt", false).unwrap();
        target.as_file().unwrap().write_all(b"ABCDEF").unwrap();
        // Create source data file with "XY"
        let mut src = TempFile::new("test_splice_from_file_src.bin", false).unwrap();
        src.as_file().unwrap().write_all(b"XY").unwrap();
        // Splice at offset 3 with file contents
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("splice")
            .arg(target.path_str()).arg("3").arg(src.path_str())
            .arg("--file")
            .output().expect("Failed to run binrw-cli write splice --file");
        let content = std::fs::read_to_string(target.path_str()).unwrap();
        // Insert "XY" at offset 3 → "ABCXYDEF"
        assert_eq!(content, "ABCXYDEF");
    }

    #[test]
    fn quadruple_splice_hello_to_front_test() {
        let mut temp_file = TempFile::new("hello_prepend_splice.test.txt", false).expect("Error creating temp file");

        // TODO: Work on this interface?
        if let Some(file) = temp_file.as_file() {
            let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
            // let actual_outputs: Vec<&str> = Vec::new();
            println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
            for i in 0..4 {
                write_hello_once(temp_file.path_str(), "0", false);
                let data = fs::read_to_string(temp_file.path_str()).expect("Unable to open test output file.");
                // TODO: String lifetime issue idk
                // actual_outputs.push(&data);
                println!("READ: {} | i={}", data, i);
                assert_eq!(data, expected_outputs[i]);
            }
            // assert_eq!(expected_outputs, actual_outputs);
        }
    }

    #[test]
    fn quadruple_splice_hello_to_eof_test() {
        let mut temp_file = TempFile::new("hello_eof_splice.test.txt", false).expect("Error creating temp file");

        if let Some(file) = temp_file.as_file() {
            let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
            println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
            for i in 0..4 {
                write_hello_once(temp_file.path_str(), "eof", true);
                let data = fs::read_to_string(temp_file.path_str()).expect("Unable to open test output file.");
                println!("READ: {} | i={}", data, i);
                assert_eq!(data, expected_outputs[i]);
            }
        }
    }
    
    #[test]
    fn check_write_overwrite_from_file() {
        // Create target file
        let mut target = TempFile::new("test_splice_overwrite_from_file.txt", false).unwrap();
        target.as_file().unwrap().write_all(b"ABCDEFGHIJ").unwrap();
        // Create source data file with "999"
        let mut src = TempFile::new("test_splice_overwrite_from_file_src.bin", false).unwrap();
        src.as_file().unwrap().write_all(b"999").unwrap();
        // Overwrite at offset 0 with file contents
        let _ = std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("write").arg("overwrite")
            .arg(target.path_str()).arg("0").arg(src.path_str())
            .arg("--file")
            .output().expect("Failed to run binrw-cli write overwrite --file");
        let content = std::fs::read_to_string(target.path_str()).unwrap();
        // Overwrite first 3 bytes → "999DEFGHIJ"
        assert_eq!(content, "999DEFGHIJ");
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