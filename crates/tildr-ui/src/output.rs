use crate::{color::Colorize, symbols::icons};

pub fn success(msg: &str) {
  println!("{}{}", icons().check.green(), msg);
}

pub fn error(msg: &str) {
  eprintln!("{}{}", icons().cross.red(), msg);
}

pub fn warn(msg: &str) {
  println!("{}{}", icons().warn.yellow(), msg);
}

pub fn info(msg: &str) {
  println!("{}{}", icons().info.cyan(), msg);
}

pub fn item(label: &str, value: &str) {
  println!("  {}{}: {}", icons().arrow.magenta(), label, value);
}

pub struct ActionLog {
  pub action: String,
  pub file: String,
}

pub fn print_actions(actions: &[ActionLog], quiet: bool) {
  println!();

  if quiet || actions.is_empty() {
    return;
  }

  let rendered_actions: Vec<String> = actions
    .iter()
    .map(|action| options_actions(&action.action))
    .collect();

  let max_action_len = rendered_actions
    .iter()
    .map(|action| visible_width(action))
    .max()
    .unwrap_or(0)
    .max(visible_width("ACTION"));

  let padding = 3;
  let width = max_action_len + padding;

  println!(
    "{}{}",
    format_column("ACTION", width),
    format_column("TARGET", width)
  );

  for (entry, action) in actions.iter().zip(rendered_actions.iter()) {
    println!(
      "{}{}",
      format_column(action, width),
      format_column(&entry.file, width)
    );
  }

  println!();
}

fn options_actions(action: &str) -> String {
  if action.contains('\u{1b}') {
    return action.to_string();
  }

  match action {
    s if s.starts_with("Would") => s.to_string(),
    "Created" => format!("{}{}", icons().check, action).green(),
    "Deleted" => format!("{}{}", icons().cross, action).red(),
    "Unlinked" => format!("{}{}", icons().warn, action).yellow(),
    "Removed" => format!("{}{}", icons().cross, action).red(),
    "Updated" => format!("{}{}", icons().check, action).green(),
    "Restored" => format!("{}{}", icons().check, action).green(),
    _ => action.normal(),
  }
}

pub fn format_column(text: &str, width: usize) -> String {
  let visible = visible_width(text);
  let padding = width.saturating_sub(visible);
  format!("{text}{}", " ".repeat(padding))
}

pub(crate) fn visible_width(text: &str) -> usize {
  strip_ansi(text).chars().count()
}

// Thank you AI for helping me with this function 'strip_ansi'.
// Suggested by Claude (claude.ai) — Anthropic. :)
fn strip_ansi(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();

  while let Some(ch) = chars.next() {
    if ch == '\u{1b}' && chars.peek() == Some(&'[') {
      chars.next();
      for ansi_char in chars.by_ref() {
        if ('@'..='~').contains(&ansi_char) {
          break;
        }
      }
      continue;
    }

    result.push(ch);
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

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
}

pub enum SummaryKind {
  Apply {
    created: usize,
    updated: usize,
    up_to_date: usize,
  },
  Remove {
    removed: usize,
    skipped: usize,
  },
  Delete {
    deleted: usize,
    skipped: usize,
  },
  Restore {
    restore: usize,
    skipped: usize,
  },
  Unlink {
    unlinked: usize,
    skipped: usize,
  },
  Add {
    added: usize,
    skipped: usize,
  },
  Move {
    moved: usize,
    skipped: usize,
  },
}

pub fn print_summary(kind: SummaryKind, dry_run: bool, quiet: bool) {
  if quiet {
    return;
  }

  let prefix = if dry_run {
    "[dry-run] ".cyan()
  } else {
    "".to_string()
  };

  match kind {
    SummaryKind::Apply {
      created,
      updated,
      up_to_date,
    } => {
      let total = created + updated;

      if total == 0 {
        println!(
          "{}{}Nothing to do ({} already up to date)",
          icons().check.green(),
          prefix,
          up_to_date
        );
      } else {
        println!(
          "{}{}{} link{} created, {} updated",
          icons().info.cyan(),
          prefix,
          created,
          if created == 1 { "" } else { "s" },
          updated
        );
      }
    }

    SummaryKind::Remove { removed, skipped } => {
      if removed == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} removed",
          icons().info.cyan(),
          prefix,
          removed,
          if removed == 1 { "" } else { "s" }
        );
      }
    }

    SummaryKind::Delete { deleted, skipped } => {
      if deleted == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} delete",
          icons().info.cyan(),
          prefix,
          deleted,
          if deleted == 1 { "" } else { "s" }
        );
      }
    }

    SummaryKind::Restore { restore, skipped } => {
      if restore == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} restore",
          icons().info.cyan(),
          prefix,
          restore,
          if restore == 1 { "" } else { "s" }
        );
      }
    }

    SummaryKind::Unlink { unlinked, skipped } => {
      if unlinked == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} unlinked",
          icons().info.cyan(),
          prefix,
          unlinked,
          if unlinked == 1 { "" } else { "s" }
        );
      }
    }

    SummaryKind::Add { added, skipped } => {
      if added == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} added",
          icons().info.cyan(),
          prefix,
          added,
          if added == 1 { "" } else { "s" }
        );
      }
    }

    SummaryKind::Move { moved, skipped } => {
      if moved == 0 {
        println!(
          "{}{}Nothing to do ({} skipped)",
          icons().check.green(),
          prefix,
          skipped
        );
      } else {
        println!(
          "{}{}{} file{} moved",
          icons().info.cyan(),
          prefix,
          moved,
          if moved == 1 { "" } else { "s" }
        );
      }
    }
  }
}
