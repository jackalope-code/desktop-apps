pub mod tempfile;

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