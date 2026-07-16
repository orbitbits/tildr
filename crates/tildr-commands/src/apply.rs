use std::fs;

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

use crate::profile::Profiles;

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
  let profiles = Profiles::load(ctx)?;

  let mut actions = Vec::new();
  let mut created = 0;
  let mut updated = 0;
  let mut up_to_date = 0;

  for entry in &entries {
    let home = ctx.home_path.join(&entry.relative);
    let file_str = entry.relative.display().to_string();
    let repo = profiles.resolve(&ctx.repo_path, &file_str);

    let exists = home.exists();
    let is_link = is_symlink(&home);

    // --- Case 1: Correct symlink ---
    if is_link && is_symlink_to(&home, &repo) {
      up_to_date += 1;

      if args.verbose && !args.quiet {
        actions.push(ActionLog {
          action: "Unchanged".to_string(),
          file: file_str,
        });
      }

      continue;
    }

    // --- Decide action type ---
    let (action_str, is_update, needs_removal) = if !exists {
      ("Created", false, false)
    } else if is_link {
      // Broken or wrong symlink → FIX automatically
      ("Updated", true, true)
    } else {
      // Regular file / dir
      if args.force {
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
      let _ = remove_file_or_dir(&home);
    }

    if let Some(parent) = home.parent() {
      fs::create_dir_all(parent)?;
    }

    create_symlink(&repo, &home)?;

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
