use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::path;
use std::fs;

#[cfg(test)]
mod write_splice_tests {
    use super::*;
    
    // TODO: Add a toggle param to make the file self-delete with RCs
    fn create_empty_test_file_from_str(filename: &str) -> &Path {
        let test_output_file_path = Path::new(filename);

        // Check if the test output file exists, delete it if it does.
        if test_output_file_path.exists() {
            fs::remove_file(test_output_file_path);
        }
        // Create new empty test file.
        {
            let f = File::create(test_output_file_path);
        }
        return test_output_file_path;
    }

    fn write__splice_data(path_str: &str, position: &str, data: &str, print_output: bool) -> String {
        let output = Command::new("./target/debug/binrw-cli.exe")
            /* TODO: Issues setting args from variables */
            .arg("write")
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
        return write_data(path_str, position, "hello", print_output);
    }

    #[ignore]
    #[test]
    fn write_large_positive_offsets() {
        assert_eq!(4, 3);
    }
    
    #[ignore]
    #[test]
    fn write_large_positive_offset_to_negative_offset() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_negative_offset_to_large_positive_offsets() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_large_positive_offset_to_eof() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_negative_offset_to_eof() {
        assert_eq!(4, 3);
    }
    
    #[ignore]
    #[test]
    fn write_negative_to_negative_ascending_pass() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_negative_to_negative_descending_fail() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_to_eof_success() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn invalid_write_offset_fail() {
        assert_eq!(4, 3);
    }

    #[ignore]
    #[test]
    fn write_verify() {
        assert_eq!(4, 3);
    }


    #[test]
    fn quadruple_splice_hello_to_front_test() {
        let test_output_file_path = create_empty_test_file_from_str("hello_prepend_splice.test.txt");

        let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
        // let actual_outputs: Vec<&str> = Vec::new();
        println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
        for i in 0..4 {
            write_hello_once(test_output_file_path.to_str().unwrap(), "0", true);
            let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
            // TODO: String lifetime issue idk
            // actual_outputs.push(&data);
            println!("READ: {} | i={}", data, i);
            assert_eq!(data, expected_outputs[i]);
        }
        // assert_eq!(expected_outputs, actual_outputs);
    }

    #[test]
    fn quadruple_splice_hello_to_eof_test() {
        let test_output_file_path = create_empty_test_file_from_str("hello_eof_splice.test.txt");

        let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
        println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
        for i in 0..4 {
            write_hello_once(test_output_file_path.to_str().unwrap(), "eof", true);
            let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
            println!("READ: {} | i={}", data, i);
            assert_eq!(data, expected_outputs[i]);
        }
    }

    #[test]
    fn count_down_to_middle() {
        let test_output_file_path = create_empty_test_file_from_str("count_down_to_middle.test.txt");
    
        let expected_outputs = vec!["55", "5445", "543345", "54322345", "5432112345"];
        println!("COUNT DOWN FROM 5 TO 1 FROM BOTH SIDES TO THE MIDDLE");
        for i in 1..6 {
            write_hello_once(test_output_file_path.to_str().unwrap(), "eof", true);
            let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
            println!("READ: {} | i={}", data, i);
            assert_eq!(data, expected_outputs[i]);
        }
    }
}