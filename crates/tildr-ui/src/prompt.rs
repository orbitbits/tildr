use console::style;
use dialoguer::theme::Theme;
use std::fmt;

pub struct MinimalTheme;

impl Theme for MinimalTheme {
  fn format_select_prompt_item(
    &self,
    f: &mut dyn fmt::Write,
    text: &str,
    active: bool,
  ) -> fmt::Result {
    if active {
      write!(f, "{} {}", style(">").cyan(), style(text).cyan())
    } else {
      write!(f, "  {}", text)
    }
  }

  fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
    write!(f, "{}", prompt) // sem prefix, sem suffix
  }

  fn format_select_prompt_selection(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    sel: &str,
  ) -> fmt::Result {
    write!(f, "{} {}", prompt, sel) // após seleção: sem ✔, sem ·
  }
}
