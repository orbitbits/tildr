#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn visible_width_ignores_ansi_sequences() {
    assert_eq!(visible_width("\x1b[32mPushed\x1b[0m"), 6);
  }

  #[test]
  fn format_column_pads_colored_text_by_visible_width() {
    let formatted = format_column("\x1b[32mPushed\x1b[0m", 12);
    assert_eq!(visible_width(&formatted), 12);
  }
}
