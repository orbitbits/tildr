use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use tildr_core::context::Context;
use tildr_fs::{
  symlink::{create_symlink, is_symlink, is_symlink_to, is_symlink_within},
  utils::remove_file_or_dir,
};
use tildr_ui::{
  output::{ActionLog, SummaryKind, print_actions, print_summary},
  warn,
};

use crate::{
  profile::Profiles,
  utils::target::{ManagedEntryProfile, effective_entries, scan_all_entries_with_profile},
};

pub struct ApplyArgs {
  pub check: bool,
  pub dry_run: bool,
  pub force: bool,
  pub verbose: bool,
  pub quiet: bool,
}

pub fn run(ctx: &Context, args: ApplyArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    warn("Repository not initialized. Run `tildr init` first.");
    return Ok(());
  }

  let entries = scan_all_entries_with_profile(ctx)?;
  let profiles = Profiles::load(ctx)?;
  let mut by_filepath: HashMap<PathBuf, Vec<ManagedEntryProfile>> = HashMap::new();
  for entry in entries {
    by_filepath
      .entry(entry.filepath.clone())
      .or_default()
      .push(entry);
  }
  let effective_by_filepath: HashMap<PathBuf, ManagedEntryProfile> =
    effective_entries(&ctx.repo_path, &profiles, &by_filepath)
      .into_iter()
      .map(|entry| (entry.filepath.clone(), entry))
      .collect();
  let files: BTreeSet<PathBuf> = by_filepath.keys().cloned().collect();

  let mut actions = Vec::new();
  let mut created = 0;
  let mut updated = 0;
  let mut removed = 0;
  let mut up_to_date = 0;
  let mut check_issues = 0;

  for file in &files {
    let file_str = file.display().to_string();
    let home = ctx.home_path.join(file);
    let Some(entry) = effective_by_filepath.get(file) else {
      if is_symlink(&home) && is_symlink_within(&home, &ctx.repo_path) {
        if args.check {
          check_issues += 1;
          actions.push(ActionLog {
            action: "Unexpected".to_string(),
            file: file_str,
          });
        } else {
          actions.push(ActionLog {
            action: if args.dry_run {
              "Would unlink".to_string()
            } else {
              "Unlinked".to_string()
            },
            file: file_str,
          });
          removed += 1;
          if !args.dry_run {
            remove_file_or_dir(&home)?;
          }
        }
      }
      continue;
    };
    let repo = &entry.repo_path;

    if !repo.exists() {
      continue;
    }

    let exists = home.exists();
    let is_link = is_symlink(&home);

    // --- Case 1: Correct symlink ---
    if is_link && is_symlink_to(&home, repo) {
      up_to_date += 1;

      if args.verbose && !args.quiet {
        actions.push(ActionLog {
          action: "Unchanged".to_string(),
          file: file_str,
        });
      }

      continue;
    }

    if args.check {
      check_issues += 1;

      let action = if is_link {
        "Broken"
      } else if !exists {
        "Missing"
      } else {
        "Conflict"
      };

      actions.push(ActionLog {
        action: action.to_string(),
        file: file_str,
      });

      continue;
    }

    // --- Decide action type ---
    let (action_str, is_update, needs_removal) = if is_link {
      ("Updated", true, true)
    } else if !exists {
      ("Created", false, false)
    } else if args.force {
      ("Updated", true, true)
    } else {
      up_to_date += 1;

      if args.verbose && !args.quiet {
        actions.push(ActionLog {
          action: "Skipped".to_string(),
          file: file_str,
        });
      }

      continue;
    };

    // --- Dry run ---
    if args.dry_run {
      let action = if is_update {
        "Would update"
      } else {
        "Would create"
      };

      actions.push(ActionLog {
        action: action.to_string(),
        file: file_str,
      });

      if is_update {
        updated += 1;
      } else {
        created += 1;
      }

      continue;
    }

    // --- Apply changes ---
    if needs_removal {
      remove_file_or_dir(&home)?;
    }

    if let Some(parent) = home.parent() {
      fs::create_dir_all(parent)?;
    }

    create_symlink(repo, &home)?;

    actions.push(ActionLog {
      action: action_str.to_string(),
      file: file_str,
    });

    if is_update {
      updated += 1;
    } else {
      created += 1;
    }
  }

  print_actions(&actions, args.quiet);

  print_summary(
    if args.check {
      SummaryKind::Check {
        checked: up_to_date + check_issues,
        issues: check_issues,
      }
    } else {
      SummaryKind::Apply {
        created,
        updated,
        removed,
        up_to_date,
      }
    },
    args.dry_run,
    args.quiet,
  );

  if args.check && check_issues > 0 {
    bail!(
      "apply check failed: {} managed file{} not correctly linked",
      check_issues,
      if check_issues == 1 { " is" } else { "s are" }
    );
  }

  Ok(())
}
