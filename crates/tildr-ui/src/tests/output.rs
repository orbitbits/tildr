use crate::output::*;

#[test]
fn strip_ansi_plain_text() {
  assert_eq!(strip_ansi("hello"), "hello");
}

#[test]
fn strip_ansi_removes_sgr_codes() {
  assert_eq!(strip_ansi("\x1b[32mPushed\x1b[0m"), "Pushed");
}

#[test]
fn strip_ansi_mixed_content() {
  assert_eq!(
    strip_ansi("\x1b[31merror\x1b[0m: file not found"),
    "error: file not found"
  );
}

#[test]
fn strip_ansi_empty_string() {
  assert_eq!(strip_ansi(""), "");
}

#[test]
fn visible_width_plain_text() {
  assert_eq!(visible_width("hello"), 5);
}

#[test]
fn visible_width_with_ansi() {
  assert_eq!(visible_width("\x1b[32mPushed\x1b[0m"), 6);
}

#[test]
fn visible_width_empty() {
  assert_eq!(visible_width(""), 0);
}

#[test]
fn format_column_shorter_than_width() {
  let result = format_column("hi", 6);
  assert_eq!(visible_width(&result), 6);
  assert!(result.ends_with("    "));
}

#[test]
fn format_column_exact_width() {
  let result = format_column("hello", 5);
  assert_eq!(visible_width(&result), 5);
}

#[test]
fn format_column_with_ansi() {
  let result = format_column("\x1b[32mhi\x1b[0m", 6);
  assert_eq!(visible_width(&result), 6);
}

#[test]
fn options_actions_created_uses_check() {
  let result = options_actions("Created");
  assert!(result.contains('\u{2714}') || result.contains('*'));
  assert!(result.contains("Created"));
}

#[test]
fn options_actions_deleted_uses_cross() {
  let result = options_actions("Deleted");
  assert!(result.contains('\u{2716}') || result.contains('x'));
  assert!(result.contains("Deleted"));
}

#[test]
fn options_actions_would_preserves_prefix() {
  let result = options_actions("Would create");
  assert_eq!(result, "Would create");
}

#[test]
fn options_actions_unknown_uses_normal() {
  let result = options_actions("Unknown");
  assert!(!result.contains('\u{2714}'));
  assert!(!result.contains('\u{2716}'));
}
