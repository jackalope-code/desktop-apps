use std::fs;
use std::fs::File;
use std::process::Command;
use std::path::Path;
use std::path::PathBuf;
use std::io;

pub struct TempFile {
  path_ref: PathBuf,
  file: Option<File>,
  keep_file: bool
}

impl TempFile {
  pub fn new(filename: &str, keep_file: bool) -> io::Result<Self> {
      let mut test_output_file_path = PathBuf::new();
      test_output_file_path.push(filename);

      // Create new empty temp file.
      let file = File::create(test_output_file_path.as_path()).expect("Could not create temp file");

      Ok(TempFile {
          path_ref: test_output_file_path,
          file: Some(file),
          keep_file
      })
  }

  pub fn as_file(&mut self) -> Option<&mut File> {
      self.file.as_mut()
  }

  pub fn path_str(&mut self) -> &str {
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