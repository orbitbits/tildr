pub trait Capitalize {
  fn capitalize(&self) -> String;
}

impl Capitalize for String {
  fn capitalize(&self) -> String {
    let mut chars = self.chars();
    match chars.next() {
      None => String::new(),
      Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
  }
}
