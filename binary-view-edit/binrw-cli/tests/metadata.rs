#[cfg(test)]
mod metadata_tests {
    use binrw_cli::utils::tempfile::TempFile;
    use std::io::Write;

    fn run_metadata_command(filename: &str) -> String {
        let bin_path = std::env::var("CARGO_BIN_EXE_binrw-cli")
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "target\\debug\\binrw-cli.exe".to_string()
                } else {
                    "target/debug/binrw-cli".to_string()
                }
            });
        let output = std::process::Command::new(bin_path)
            .arg("metadata")
            .arg(filename)
            .output()
            .expect("Failed to run binrw-cli metadata");
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[test]
    fn read_metadata_alphabet() {
        let output = run_metadata_command("tests/data/alphabet.txt");
        // metadata prints the file size (26 bytes for alphabet.txt)
        assert!(output.contains("26"), "Expected metadata to contain size 26, got: {}", output);
    }

    #[test]
    fn read_metadata_temp_file() {
        let mut file = TempFile::new("test_metadata.bin", false).unwrap();
        let data = vec![0xABu8; 512];
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_metadata_command(file.path_str());
        assert!(output.contains("512"), "Expected metadata to contain size 512, got: {}", output);
    }
}