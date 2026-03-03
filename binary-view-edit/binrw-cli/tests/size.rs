#[cfg(test)]
mod size_tests {
    use std::process::Command;
    use binrw_cli::utils::tempfile::TempFile;
    use std::io::Write;

    fn run_size_command(filename: &str) -> String {
        let output = Command::new("target/debug/binrw-cli.exe")
            .arg("size")
            .arg(filename)
            .output()
            .expect("Failed to run binrw-cli size");
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[test]
    fn check_size_command() {
        // alphabet.txt is 26 bytes
        let output = run_size_command("tests/data/alphabet.txt");
        assert!(output.contains("26"), "Expected size 26, got: {}", output);
    }

    #[test]
    fn check_large_file_size() {
        let mut file = TempFile::new("test_large_size.bin", false).unwrap();
        let data = vec![0u8; 10000];
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_size_command(file.path_str());
        assert!(output.contains("10000"), "Expected size 10000, got: {}", output);
    }
}