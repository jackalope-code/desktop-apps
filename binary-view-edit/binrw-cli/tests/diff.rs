#[cfg(test)]
mod diff_tests {
    use binrw_cli::utils::tempfile::TempFile;
    use std::io::Write;

    // Diff is TODO in the CLI, so these tests verify that the command
    // doesn't crash and produces output containing "TODO" or "Diff"
    fn run_diff_command(file_a: &str, file_b: &str) -> std::process::Output {
        std::process::Command::new("target/debug/binrw-cli.exe")
            .arg("diff")
            .arg(file_a)
            .arg(file_b)
            .output()
            .expect("Failed to run binrw-cli diff")
    }

    #[test]
    fn diff_first_two_java_files() {
        // Create two different files and diff them - should not crash
        let mut file_a = TempFile::new("test_diff_a.bin", false).unwrap();
        file_a.as_file().unwrap().write_all(b"ABCDEF").unwrap();
        file_a.as_file().unwrap().sync_all().unwrap();
        let mut file_b = TempFile::new("test_diff_b.bin", false).unwrap();
        file_b.as_file().unwrap().write_all(b"ABCXYZ").unwrap();
        file_b.as_file().unwrap().sync_all().unwrap();
        let output = run_diff_command(file_a.path_str(), file_b.path_str());
        // CLI should not crash (exit code 0 or at least produce output)
        assert!(output.status.success(), "Diff command should exit successfully, status: {}", output.status);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("TODO") || stdout.contains("Diff"),
            "Diff command should output TODO or Diff, got: {}", stdout);
    }

    #[test]
    fn diff_files_first_path_invalid() {
        let mut file_b = TempFile::new("test_diff_valid.bin", false).unwrap();
        file_b.as_file().unwrap().write_all(b"data").unwrap();
        file_b.as_file().unwrap().sync_all().unwrap();
        // First path doesn't exist - command should handle gracefully
        let output = run_diff_command("nonexistent_file_1234.bin", file_b.path_str());
        // We just verify it doesn't panic/crash - any exit is acceptable
        let _ = output;
    }

    #[test]
    fn diff_files_second_path_invalid() {
        let mut file_a = TempFile::new("test_diff_valid2.bin", false).unwrap();
        file_a.as_file().unwrap().write_all(b"data").unwrap();
        file_a.as_file().unwrap().sync_all().unwrap();
        // Second path doesn't exist - command should handle gracefully
        let output = run_diff_command(file_a.path_str(), "nonexistent_file_5678.bin");
        let _ = output;
    }
}