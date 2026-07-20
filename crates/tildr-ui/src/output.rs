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

pub(crate) fn options_actions(action: &str) -> String {
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
    "Missing" => format!("{}{}", icons().cross, action).red(),
    "Broken" => format!("{}{}", icons().cross, action).red(),
    "Conflict" => format!("{}{}", icons().warn, action).yellow(),
    "Unexpected" => format!("{}{}", icons().warn, action).yellow(),
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
pub(crate) fn strip_ansi(text: &str) -> String {
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

pub enum SummaryKind {
  Apply {
    created: usize,
    updated: usize,
    removed: usize,
    up_to_date: usize,
  },
  Check {
    checked: usize,
    issues: usize,
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
      removed,
      up_to_date,
    } => {
      let total = created + updated + removed;

      if total == 0 {
        println!(
          "{}{}Nothing to do ({} already up to date)",
          icons().check.green(),
          prefix,
          up_to_date
        );
      } else {
        println!(
          "{}{}{} link{} created, {} updated, {} removed",
          icons().info.cyan(),
          prefix,
          created,
          if created == 1 { "" } else { "s" },
          updated,
          removed
        );
      }
    }

    SummaryKind::Check { checked, issues } => {
      if issues == 0 {
        println!(
          "{}All managed files are correctly linked ({} checked)",
          icons().check.green(),
          checked
        );
      } else {
        println!(
          "{}{} issue{} found ({} checked)",
          icons().warn.yellow(),
          issues,
          if issues == 1 { "" } else { "s" },
          checked
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
