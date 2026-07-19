use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tildr_core::context::Context;
use tildr_fs::{
  symlink::{create_symlink, is_symlink, is_symlink_to},
  utils::remove_file_or_dir,
};
use tildr_repo::scatildr_repo;
use tildr_ui::{
  output::{ActionLog, SummaryKind, print_actions, print_summary},
  warn,
};

pub struct ApplyArgs {
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

  let entries = scatildr_repo(&ctx.repo_path)?;

  // Build a map of logical path -> best repo path.
  // Prefer profile-specific entries over root-level (legacy) entries.
  let mut file_map: HashMap<String, PathBuf> = HashMap::new();
  for entry in entries {
    let logical = entry.relative.to_string_lossy().to_string();
    // If we already have a path for this logical file, prefer the one that
    // is under profiles/<name>/ (non-root) over root-level legacy files.
    let is_legacy_root = entry.profile == "default"
      && !entry
        .repo_path
        .starts_with(ctx.repo_path.join("profiles/default"));
    let existing_is_legacy = file_map
      .get(&logical)
      .map(|p| p.starts_with(ctx.repo_path.join("profiles/default")))
      .unwrap_or(true); // default to true so new non-legacy wins

    if !is_legacy_root || existing_is_legacy {
      file_map.insert(logical, entry.repo_path);
    }
  }

  let mut files: Vec<String> = file_map.keys().cloned().collect();
  files.sort();

  let mut actions = Vec::new();
  let mut created = 0;
  let mut updated = 0;
  let mut up_to_date = 0;

  for file_str in &files {
    let home = ctx.home_path.join(file_str);
    let repo = file_map[file_str].clone();

    if !repo.exists() {
      continue;
    }

    let exists = home.exists();
    let is_link = is_symlink(&home);

    // --- Case 1: Correct symlink ---
    if is_link && is_symlink_to(&home, &repo) {
      up_to_date += 1;

      if args.verbose && !args.quiet {
        actions.push(ActionLog {
          action: "Unchanged".to_string(),
          file: file_str.clone(),
        });
      }

      continue;
    }

    // --- Decide action type ---
    let (action_str, is_update, needs_removal) = if is_link {
      ("Updated", true, true)
    } else if !exists {
      ("Created", false, false)
    } else {
      if args.force {
        ("Updated", true, true)
      } else {
        up_to_date += 1;

        if args.verbose && !args.quiet {
          actions.push(ActionLog {
            action: "Skipped".to_string(),
            file: file_str.clone(),
          });
        }

        continue;
      }
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
        file: file_str.clone(),
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
      let _ = remove_file_or_dir(&home);
    }

    if let Some(parent) = home.parent() {
      fs::create_dir_all(parent)?;
    }

    create_symlink(&repo, &home)?;

    actions.push(ActionLog {
      action: action_str.to_string(),
      file: file_str.clone(),
    });

    if is_update {
      updated += 1;
    } else {
      created += 1;
    }
  }

  print_actions(&actions, args.quiet);

  print_summary(
    SummaryKind::Apply {
      created,
      updated,
      up_to_date,
    },
    args.dry_run,
    args.quiet,
  );

  Ok(())
}
