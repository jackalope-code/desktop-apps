#[cfg(test)]
mod file_id_tests {
    use binrw_cli::utils::tempfile::TempFile;
    use std::io::Write;

    fn run_type_command(filename: &str) -> String {
        let bin_path = std::env::var("CARGO_BIN_EXE_binrw-cli")
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "target\\debug\\binrw-cli.exe".to_string()
                } else {
                    "target/debug/binrw-cli".to_string()
                }
            });
        let output = std::process::Command::new(bin_path)
            .arg("type")
            .arg(filename)
            .output()
            .expect("Failed to run binrw-cli type");
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[test]
    fn id_jpg() {
        // JPG magic bytes: FF D8 ... FF D9
        let mut file = TempFile::new("test_id.jpg", false).unwrap();
        let mut data = vec![0xFFu8, 0xD8, 0xFF, 0xE0];
        data.extend_from_slice(&[0u8; 100]);
        data.extend_from_slice(&[0xFF, 0xD9]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("jpg"), "Expected jpg, got: {}", output);
    }

    #[test]
    fn id_bmp() {
        // BMP magic bytes: 42 4D
        let mut file = TempFile::new("test_id.bmp", false).unwrap();
        let mut data = vec![0x42u8, 0x4D, 0x00, 0x00];
        data.extend_from_slice(&[0u8; 100]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("bmp"), "Expected bmp, got: {}", output);
    }

    #[test]
    fn id_gif() {
        // GIF magic bytes: 47 49 46 38
        let mut file = TempFile::new("test_id.gif", false).unwrap();
        let mut data = vec![0x47u8, 0x49, 0x46, 0x38];
        data.extend_from_slice(&[0u8; 100]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("gif"), "Expected gif, got: {}", output);
    }

    #[test]
    fn id_png() {
        // PNG magic bytes: 89 50 4E 47
        let mut file = TempFile::new("test_id.png", false).unwrap();
        let mut data = vec![0x89u8, 0x50, 0x4E, 0x47];
        data.extend_from_slice(&[0u8; 100]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("png"), "Expected png, got: {}", output);
    }

    #[test]
    fn id_exe() {
        // EXE/PE magic bytes: 4D 5A
        let mut file = TempFile::new("test_id.exe_file", false).unwrap();
        let mut data = vec![0x4Du8, 0x5A, 0x00, 0x00];
        data.extend_from_slice(&[0u8; 100]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("exe"), "Expected exe, got: {}", output);
    }

    #[test]
    fn id_class() {
        // Java class magic bytes: CA FE BA BE
        let mut file = TempFile::new("test_id.class", false).unwrap();
        let mut data = vec![0xCAu8, 0xFE, 0xBA, 0xBE];
        data.extend_from_slice(&[0u8; 100]);
        file.as_file().unwrap().write_all(&data).unwrap();
        file.as_file().unwrap().sync_all().unwrap();
        let output = run_type_command(file.path_str());
        assert!(output.contains("class"), "Expected class, got: {}", output);
    }
}