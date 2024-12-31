#[macro_export]
macro_rules! cmd {
  ( $($x: expr), * ) => {
    {
      let mut cmd;
      let mut i = 0;
      $(
        if i == 0 {
          cmd = Command::new(std::ffi::OsStr::new($x));
        } else {
          cmd.arg(std::ffi::OsStr::new($x));
        }
        i += 1;
      )*
      cmd
    }
  };
}