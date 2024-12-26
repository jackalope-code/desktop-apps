
mod utilities {

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
        let file = File::create(test_output_file_path.as_path()).expect("Could not create temp file");
  
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
        println!("TempFile dropping out of scope!!!");
        if !self.keep_file {
            println!("Deleting file!!!");
            fs::remove_file(self.path_ref.as_path());
        }
    }
  }
}