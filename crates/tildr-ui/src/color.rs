use std::{env, io::IsTerminal};

pub trait Colorize {
  fn green(&self) -> String;
  fn yellow(&self) -> String;
  fn red(&self) -> String;
  fn cyan(&self) -> String;
  fn magenta(&self) -> String;
  fn normal(&self) -> String;
  fn bold(&self) -> String;
}

impl<T: AsRef<str>> Colorize for T {
  fn green(&self) -> String {
    colorize(self.as_ref(), 32)
  }

  fn yellow(&self) -> String {
    colorize(self.as_ref(), 33)
  }

  fn red(&self) -> String {
    colorize(self.as_ref(), 31)
  }

  fn cyan(&self) -> String {
    colorize(self.as_ref(), 36)
  }

  fn magenta(&self) -> String {
    colorize(self.as_ref(), 35)
  }
  fn normal(&self) -> String {
    colorize(self.as_ref(), 0)
  }
  fn bold(&self) -> String {
    colorize(self.as_ref(), 1)
  }
}

fn colorize(text: &str, code: u8) -> String {
  if !use_color() {
    return text.to_string();
  }

  format!("\x1b[{}m{}\x1b[0m", code, text)
}

fn use_color() -> bool {
  // NO_COLOR standard (https://no-color.org/)
  if env::var("NO_COLOR").is_ok() {
    return false;
  }

  // It only uses color if it's a terminal
  std::io::stdout().is_terminal()
}
