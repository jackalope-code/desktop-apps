#[cfg(test)]
mod diff_tests {
    use super::*;

    // fn diff() -> String {
    //     //  binrw write splice file_path position hello
    //     let output = Command::new("./target/debug/binrw-cli.exe")
    //         .arg("write")
    //         .arg("splice")
    //         .arg(path_str)
    //         .arg(position)
    //         .arg(data)
    //         .output()
    //         .expect("Failed to execute binrw-cli.");
    //     let stdout = String::from_utf8_lossy(&output.stdout);
    //     let stderr = String::from_utf8_lossy(&output.stderr);
    //     if print_output {
    //         println!("STDOUT: {}", stdout);
    //         println!("STDERR: {}", stderr);
    //     }
    //     return stdout.to_string();
    // }

    #[test]
    fn diff_first_two_java_files() {
        assert_eq!(4, 4);
    }

    #[test]
    fn diff_files_first_path_invalid() {
        assert_eq!(4, 4);
    }

    #[test]
    fn diff_files_second_path_invalid() {
        assert_eq!(4, 4);
    }
}