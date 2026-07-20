use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use console::style;
use tildr_core::context::Context;
use walkdir::WalkDir;

use crate::profile::{COMMON_PROFILE, profile_dir};
use crate::utils::auto_commit::auto_commit;

pub struct CleanArgs {
  pub dry_run: bool,
  pub quiet: bool,
}

pub fn run(ctx: &Context, args: CleanArgs) -> Result<()> {
  let removed = clean_empty_profile_dirs(ctx, args.dry_run)?;

  if !args.quiet {
    for dir in &removed {
      let label = if args.dry_run {
        style("Would remove:").cyan()
      } else {
        style("Removed:").green()
      };
      println!("  {} {}", label, dir.display());
    }

    if removed.is_empty() {
      println!("{}", style("Nothing to clean.").dim());
    } else if args.dry_run {
      println!(
        "\n{} {} empty directories would be removed",
        style("Would clean:").cyan(),
        removed.len()
      );
    } else {
      println!(
        "\n{} {} empty directories removed",
        style("Cleaned:").green().bold(),
        removed.len()
      );
    }
  }

  if !args.dry_run && !removed.is_empty() {
    auto_commit(ctx, &format!("clean {} empty directories", removed.len()));
  }

  Ok(())
}

pub(crate) fn clean_empty_profile_dirs(ctx: &Context, dry_run: bool) -> Result<Vec<PathBuf>> {
  let mut removed = Vec::new();
  let roots = cleanup_roots(ctx);

  for root in roots {
    collect_empty_dirs(ctx, &root, dry_run, &mut removed)?;
  }

  removed.sort();
  Ok(removed)
}

fn cleanup_roots(ctx: &Context) -> Vec<PathBuf> {
  let mut roots = Vec::new();

  let common_dir = profile_dir(&ctx.repo_path, COMMON_PROFILE);
  if common_dir.is_dir() {
    roots.push(common_dir);
  }

  let profiles_dir = ctx.repo_path.join("profiles");
  if let Ok(entries) = fs::read_dir(&profiles_dir) {
    for entry in entries.filter_map(|entry| entry.ok()) {
      let path = entry.path();
      if path.is_dir() {
        roots.push(path);
      }
    }
  }

  roots
}

fn collect_empty_dirs(
  ctx: &Context,
  root: &Path,
  dry_run: bool,
  removed: &mut Vec<PathBuf>,
) -> Result<()> {
  let mut dirs: Vec<PathBuf> = WalkDir::new(root)
    .min_depth(1)
    .into_iter()
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.file_type().is_dir())
    .map(|entry| entry.into_path())
    .collect();

  dirs.sort_by_key(|path| std::cmp::Reverse(path.components().count()));

  for dir in dirs {
    if is_empty_dir(&dir)? {
      let display_path = dir
        .strip_prefix(&ctx.repo_path)
        .unwrap_or(&dir)
        .to_path_buf();
      if !dry_run {
        fs::remove_dir(&dir)?;
      }
      removed.push(display_path);
    }
  }

  Ok(())
}

fn is_empty_dir(path: &Path) -> Result<bool> {
  Ok(path.is_dir() && fs::read_dir(path)?.next().is_none())
}
