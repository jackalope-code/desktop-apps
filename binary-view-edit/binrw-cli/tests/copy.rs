#[cfg(test)]
mod copy_tests {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use std::path::Path;
    use sha2::{Sha256, Digest};
    use binrw_cli::utils::tempfile::TempFile;
    use std::env;
    use std::path::PathBuf;

    fn file_hash<P: AsRef<std::path::Path>>(path: P) -> String {
        let data = std::fs::read(&path).expect(&format!("Could not read file: {:?}", path.as_ref()));
        let mut hasher = sha2::Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn run_copy_command(src: &str, dest: &str) {
        let bin_path = std::env::var("CARGO_BIN_EXE_binrw-cli")
            .unwrap_or_else(|_| {
                // Fallback to default build path if env var is missing
                if cfg!(windows) {
                    "target\\debug\\binrw-cli.exe".to_string()
                } else {
                    "target/debug/binrw-cli".to_string()
                }
            });
        let status = std::process::Command::new(bin_path)
            .arg("copy")
            .arg(src)
            .arg(dest)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("Failed to run binrw-cli copy");
        assert!(status.success(), "binrw-cli copy command failed");
    }

    #[test]
    fn copy_file_and_compare_hash() {
        let mut src = TempFile::new("test_copy_src.bin", false).unwrap();
        src.as_file().unwrap().write_all(b"Some binary data\x00\x01\x02").unwrap();
        let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_copy_src.bin");
        let dest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_copy_dest.bin");
        run_copy_command(src_path.to_str().unwrap(), dest_path.to_str().unwrap());
        let src_hash = file_hash(&src_path);
        let dest_hash = file_hash(&dest_path);
        assert_eq!(src_hash, dest_hash, "Hashes of source and copied file should match");
        let _ = std::fs::remove_file(&dest_path);
        let _ = std::fs::remove_file(&src_path);
    }

    #[test]
    fn copy_large_file_and_compare_hash() {
        let mut src = TempFile::new("test_copy_large_src.bin", false).unwrap();
        let data = vec![0u8; 1024 * 1024]; // 1MB of zeros
        src.as_file().unwrap().write_all(&data).unwrap();
        let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_copy_large_src.bin");
        let dest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_copy_large_dest.bin");
        run_copy_command(src_path.to_str().unwrap(), dest_path.to_str().unwrap());
        let src_hash = file_hash(&src_path);
        let dest_hash = file_hash(&dest_path);
        assert_eq!(src_hash, dest_hash, "Hashes of large source and copied file should match");
        let _ = std::fs::remove_file(&dest_path);
        let _ = std::fs::remove_file(&src_path);
    }

    #[test]
    fn copy_test_binary_and_compare_hash() {
        let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/test_binary.bin");
        let dest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_binary_copy.bin");
        run_copy_command(src_path.to_str().unwrap(), dest_path.to_str().unwrap());
        let src_hash = file_hash(&src_path);
        let dest_hash = file_hash(&dest_path);
        assert_eq!(src_hash, dest_hash, "Hashes of test binary and copied file should match");
        let _ = std::fs::remove_file(&dest_path);
    }
}
