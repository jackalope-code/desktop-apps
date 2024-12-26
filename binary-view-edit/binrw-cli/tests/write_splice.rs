use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io;

#[cfg(test)]
mod write_splice_tests {
    use super::*;

    struct TempFile {
        path_ref: PathBuf,
        file: Option<File>,
        keep_file: bool
    }

    impl TempFile {
        fn new(filename: &str, keep_file: bool) -> io::Result<Self> {
            let mut test_output_file_path = PathBuf::new();
            test_output_file_path.push(filename);

            // Create new empty test file.
            let file = File::create(test_output_file_path.as_path()).expect("Could not create test file");
    
            Ok(TempFile {
                path_ref: test_output_file_path,
                file: Some(file),
                keep_file
            })
        }

        fn as_file(&mut self) -> Option<&mut File> {
            self.file.as_mut()
        }

        fn path_str(&mut self) -> &str {
            return self.path_ref.to_str().unwrap()
        }

        // fn path(&mut self) -> &Path {
        //     return self.path_ref
        // }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            println!("TestFileRef dropping out of scope!!!");
            if !self.keep_file {
                println!("Deleting file!!!");
                fs::remove_file(self.path_ref.as_path());
            }
        }
    }

    // TODO: Add a RC for lifetime and a toggle param to make the file self-delete
    // fn create_empty_test_file_from_str(filename: &str, keep_file: bool) -> TestFileRef {
    //     let test_output_file_path = Path::new(filename);

    //     // Check if the test output file exists, delete it if it does.
    //     if test_output_file_path.exists() {
    //         fs::remove_file(test_output_file_path);
    //     }
    //     // Create new empty test file.
    //     let f = File::create(test_output_file_path).expect("Could not create test file");

    //     return TestFileRef {
    //         path: test_output_file_path,
    //         keep_file
    //     };
    // }
    
    // // TODO: Add a RC for lifetime and a toggle param to make the file self-delete
    // fn create_empty_test_file(filename: &str, keep_file: bool) -> TestFileRef {
    //     let test_output_file_path = Path::new(filename);

    //     // Check if the test output file exists, delete it if it does.
    //     if test_output_file_path.exists() {
    //         fs::remove_file(test_output_file_path);
    //     }
    //     // Create new empty test file.
    //     let f = File::create(test_output_file_path).expect("Could not create test file");

    //     return TestFileRef {
    //         file: f,
    //         path: test_output_file_path,
    //         keep_file
    //     };
    // }

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
    
    
    #[ignore]
    #[test]
    fn check_write_splice() {

    }
    
    #[ignore]
    #[test]
    fn check_write_splice_from_file() {

    }

    #[test]
    fn quadruple_splice_hello_to_front_test() {
        // let TestFileRef {path, ..} = create_empty_test_file_from_str("hello_prepend_splice.test.txt", false);
        let mut temp_file = TempFile::new("hello_prepend_splice.test.txt", false).expect("Error creating temp file");

        if let Some(file) = temp_file.as_file() {
            // let test_output_file_path = path;
            let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
            // let actual_outputs: Vec<&str> = Vec::new();
            println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
            for i in 0..4 {
                write_hello_once(temp_file.path_str(), "0", false);
                let data = fs::read_to_string(temp_file.path_ref.as_path()).expect("Unable to open test output file.");
                // TODO: String lifetime issue idk
                // actual_outputs.push(&data);
                println!("READ: {} | i={}", data, i);
                assert_eq!(data, expected_outputs[i]);
            }
            // assert_eq!(expected_outputs, actual_outputs);
        }
    }

    // #[ignore]
    // #[test]
    // fn quadruple_splice_hello_to_eof_test() {
    //     // let TestFileRef {path, ..} = create_empty_test_file_from_str("hello_eof_splice.test.txt", false);
    //     // let test_output_file_path = path;

    //     let expected_outputs = vec!["hello", "hellohello", "hellohellohello", "hellohellohellohello"];
    //     println!("WRITE \"HELLO\" TO FILE FOUR TIMES.");
    //     for i in 0..4 {
    //         write_hello_once(test_output_file_path.to_str().unwrap(), "eof", true);
    //         let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
    //         println!("READ: {} | i={}", data, i);
    //         assert_eq!(data, expected_outputs[i]);
    //     }
    // }

    // #[ignore]
    // #[test]
    // fn count_down_to_middle() {
    //     // let TestFileRef {path, ..} = create_empty_test_file_from_str("count_down_to_middle.test.txt", false);
    //     // let test_output_file_path = path;
    
    //     let expected_outputs = vec!["55", "5445", "543345", "54322345", "5432112345"];
    //     println!("COUNT DOWN FROM 5 TO 1 FROM BOTH SIDES TO THE MIDDLE");
    //     // binrw write splice filename position data
    //     // Each loop step:
    //     // Append i to both sides (0 and eof)
    //     // Check
    //     for i in 1..6 { // TODO: Loop backwards
    //         println!("{}", i)
    //         // write_splice_data(test_output_file_path.to_str().unwrap(), "eof", true);
    //         // let data = fs::read_to_string(test_output_file_path).expect("Unable to open test output file.");
    //         // println!("READ: {} | i={}", data, i);
    //         // assert_eq!(data, expected_outputs[i]);
    //     }
    // }
    
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