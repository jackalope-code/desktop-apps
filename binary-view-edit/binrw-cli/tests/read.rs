use std::process::Command;

#[cfg(test)]
mod read_tests {
    use super::*;

    fn read_position(path_str: &str, position_arg: &str, num_bytes_arg: &str) -> String {
        let output = Command::new("./target/debug/binrw-cli.exe")
            /* TODO: Issues setting args from variables */
            .arg("read")
            .arg(path_str)
            .arg(position_arg)
            .arg(num_bytes_arg)
            .output()
            .expect("Failed to execute binrw-cli.");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("STDOUT: {}", stdout);
        println!("STDERR: {}", stderr);
        return stdout.to_string();
    }

    #[ignore]
    #[test]
    fn read_large_positive_offsets() {
        assert_eq!(4, 4);
    }
    
    #[ignore]
    #[test]
    fn read_large_positive_offset_to_negative_offset() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn read_negative_offset_to_large_positive_offsets() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn read_large_positive_offset_to_eof() {
        assert_eq!(4, 4);
    }
    
    
    #[test]
    fn read_negative_offset_to_eof() {
        // TODO: copy/pasted here until i can figure out modules
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
        let result = read_position("tests/data/alphabet.txt", "-4", "eof");
        assert_eq!(format!("Read\n{}\n", parse_hex_data("WXYZ".as_bytes().to_vec(), false).join(" ")), result);
    }
    
    #[ignore]
    #[test]
    fn read_negative_to_negative_ascending_pass() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn read_negative_to_negative_descending_fail() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn read_eof_to_offset_fail() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn invalid_read_offset_parse_fail() {
        assert_eq!(4, 4);
    }
}