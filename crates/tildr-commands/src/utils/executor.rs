use anyhow::Result;
use tildr_repo::ManagedEntry;
use tildr_ui::output::ActionLog;

/// Generic executor for processing entries with dry-run support.
pub fn execute_entries<F>(
  entries: Vec<ManagedEntry>,
  dry_run: bool,
  action_label: &str,
  dry_label: &str,
  mut op: F,
) -> Result<(usize, usize, Vec<ActionLog>)>
where
  F: FnMut(&ManagedEntry) -> Result<bool>,
{
  let mut actions = Vec::new();
  let mut success = 0;
  let mut skipped = 0;

  for entry in entries {
    let file = entry.relative.display().to_string();

    if dry_run {
      actions.push(ActionLog {
        action: dry_label.to_string(),
        file,
      });
      success += 1;
      continue;
    }

    if op(&entry)? {
      success += 1;
      actions.push(ActionLog {
        action: action_label.to_string(),
        file,
      });
    } else {
      skipped += 1;
    }
  }

  Ok((success, skipped, actions))
}
