use anyhow::{Result, bail};
use std::{fs, path::PathBuf};
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};
use tildr_fs::{
  symlink::{create_symlink, is_symlink},
  utils::remove_file_or_dir,
};
use tildr_ui::{
  color::Colorize,
  icons,
  output::{ActionLog, SummaryKind, print_actions, print_summary},
  prompt::MinimalTheme,
};

use crate::utils::{
  auto_commit::auto_commit_dry_run,
  target::{ManagedEntryProfile, scan_all_entries_with_profile},
};

pub struct MvArgs {
  pub source: Option<String>,
  pub dest: Option<String>,
  pub dry_run: bool,
  pub quiet: bool,
}

fn find_entry(ctx: &Context, target: &str) -> Result<ManagedEntryProfile> {
  let entries = scan_all_entries_with_profile(ctx)?;
  entries
    .into_iter()
    .find(|e| e.filepath.to_string_lossy() == target)
    .ok_or_else(|| anyhow::anyhow!("File is not managed by tildr: {}", target))
}

pub fn run(ctx: &Context, args: MvArgs) -> Result<()> {
  // --- Resolve source ---
  let source_rel = match args.source {
    Some(ref s) => PathBuf::from(s),
    None => {
      let picked = pick::target(
        ctx,
        None,
        true,
        Some("Select a file\n-------------\n"),
        PickMode::Managed,
      )?;
      picked
        .strip_prefix(&ctx.repo_path)
        .unwrap_or(&picked)
        .to_path_buf()
    }
  };

  let entry = find_entry(ctx, &source_rel.to_string_lossy())?;
  let source_repo = entry.repo_path.clone();
  let profile_name = entry.profile.clone();

  // --- Resolve dest ---
  let dest_input = match args.dest {
    Some(ref d) => d.clone(),
    None => {
      let title = format!(
        "{} {}\n--------------\n",
        "File selected:".cyan(),
        source_rel.display()
      );
      let legend = format!("\n{} {}\n", "Actions:".bold(), "ctrl+c: cancel".magenta());
      println!("{}", legend);
      println!("{}", title);
      let input: String = dialoguer::Input::with_theme(&MinimalTheme)
        .with_prompt("New (path | name)".bold())
        .interact_text()?;
      input
    }
  };

  // If the user typed only a filename (no directory separator),
  // keep the original directory.
  let dest_rel = {
    let dest_path = PathBuf::from(&dest_input);
    if dest_path.components().count() == 1 {
      match source_rel.parent() {
        Some(parent) if parent != std::path::Path::new("") => parent.join(&dest_path),
        _ => dest_path,
      }
    } else {
      dest_path
    }
  };

  let dest_repo = ctx
    .repo_path
    .join("profiles")
    .join(&profile_name)
    .join(&dest_rel);
  let source_home = ctx.home_path.join(&source_rel);
  let dest_home = ctx.home_path.join(&dest_rel);

  // --- Validations ---
  if source_rel == dest_rel {
    bail!(
      "Source and destination are the same: {}",
      source_rel.display()
    );
  }

  if dest_repo.exists() {
    bail!("Destination already exists in repo: {}", dest_rel.display());
  }

  // --- Dry run ---
  if args.dry_run {
    let actions = vec![ActionLog {
      action: "Would move".to_string(),
      file: format!("{} → {}", source_rel.display(), dest_rel.display()),
    }];
    print_actions(&actions, args.quiet);
    print_summary(
      SummaryKind::Move {
        moved: 1,
        skipped: 0,
      },
      true,
      args.quiet,
    );
    return Ok(());
  }

  // --- Create parent dirs in repo if needed ---
  if let Some(parent) = dest_repo.parent() {
    fs::create_dir_all(parent)?;
  }

  // --- Move file in repo ---
  fs::rename(&source_repo, &dest_repo)?;

  // --- Remove old symlink in HOME ---
  if is_symlink(&source_home) {
    remove_file_or_dir(&source_home)?;
  }

  // --- Create parent dirs in HOME if needed ---
  if let Some(parent) = dest_home.parent() {
    fs::create_dir_all(parent)?;
  }

  // --- Create new symlink in HOME ---
  create_symlink(&dest_repo, &dest_home)?;

  // --- Output ---
  let actions = vec![ActionLog {
    action: format!("{}Moved", icons().check).green(),
    file: format!("{} → {}", source_rel.display(), dest_rel.display()),
  }];
  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Move {
      moved: 1,
      skipped: 0,
    },
    false,
    args.quiet,
  );

  auto_commit_dry_run(
    ctx,
    &format!("mv {} {}", source_rel.display(), dest_rel.display()),
    args.dry_run,
  );

  Ok(())
}
