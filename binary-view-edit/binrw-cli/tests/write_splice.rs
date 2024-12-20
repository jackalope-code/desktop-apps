use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::fs;

#[cfg(test)]
mod write_splice_tests {
    use super::*;

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

    // TODO: WRITE THIS FIRST
    #[test]
    fn quadruple_splice_hello_to_front_test() {
        let test_output_file_path = Path::new("hello.test.txt");
        fn write_hello_once(print_output: bool) -> String {
            let output = Command::new("./target/debug/binrw-cli.exe")
                /* TODO: Issues setting args from variables */
                .args(["write", "hello.test.txt", "0", "hello"])
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
        // Check if the test output file exists, delete it if it does.
        if test_output_file_path.exists() {
            fs::remove_file(test_output_file_path);
        }
        // Create new empty test file once before looping.
        {
            let f = File::create(test_output_file_path);
        }
        // TODO: Check file once every time after running write_hello_once, for a total of four times.
        let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
        // let actual_outputs: Vec<&str> = Vec::new();
        println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
        for i in 0..4 {
            write_hello_once(true);
            let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
            // TODO: String lifetime issue idk
            // actual_outputs.push(&data);
            println!("READ: {} | i={}", data, i);
            assert_eq!(data, expected_outputs[i]);
        }
        // assert_eq!(expected_outputs, actual_outputs);
    }

    // TODO: WRITE THIS NEXT
    #[ignore]
    #[test]
    fn quadruple_splice_hello_to_eof_test() {
        assert_eq!(4, 3);
    }
}