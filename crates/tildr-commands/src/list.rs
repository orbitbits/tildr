use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};
use tildr_core::context::Context;
use tildr_fs::symlink::create_symlink;
use tildr_repo::ManagedEntry;
use tildr_ui::info;
use tildr_utils::fs::format_size;

#[derive(Debug, Serialize, Deserialize)]
struct ExportFile {
  version: u32,
  files: Vec<String>,
}

pub struct ListArgs {
  pub tree: bool,
  pub long: bool,
  pub export: Option<String>,
  pub import: Option<String>,
}

pub fn run(ctx: &Context, args: ListArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    info("Repository not initialized. Run `tildr init` first.");
    return Ok(());
  }

  if let Some(ref path) = args.export {
    return export_to_file(ctx, path);
  }

  if let Some(ref path) = args.import {
    return import_from_file(ctx, path);
  }

  if args.tree {
    print_tree(&ctx.repo_path)?;
    return Ok(());
  }

  let entries = tildr_repo::scatildr_repo(&ctx.repo_path)?;

  if entries.is_empty() {
    info("No managed files. Run `tildr add <file>` to start.");
    return Ok(());
  }

  let count = entries.len();

  if args.long {
    print_long(&entries)?;
  } else {
    for entry in &entries {
      println!("{}", entry.relative.display());
    }
  }

  println!("\n{} file(s) managed", count);

  Ok(())
}

fn export_to_file(ctx: &Context, path: &str) -> Result<()> {
  let entries = tildr_repo::scatildr_repo(&ctx.repo_path)?;

  if entries.is_empty() {
    info("No managed files to export.");
    return Ok(());
  }

  let files: Vec<String> = entries
    .iter()
    .map(|e| e.relative.display().to_string())
    .collect();

  let export = ExportFile { version: 1, files };

  let json = serde_json::to_string_pretty(&export).context("Failed to serialize export")?;
  fs::write(path, &json).context("Failed to write export file")?;

  println!("Exported {} file(s) to {}", entries.len(), path);
  Ok(())
}

fn import_from_file(ctx: &Context, path: &str) -> Result<()> {
  let content = fs::read_to_string(path).context("Failed to read import file")?;
  let export: ExportFile =
    serde_json::from_str(&content).context("Failed to parse import file (invalid JSON)")?;

  if export.version != 1 {
    anyhow::bail!("Unsupported export version: {}", export.version);
  }

  if export.files.is_empty() {
    info("No files in import file.");
    return Ok(());
  }

  let mut created = 0u32;
  let mut skipped = 0u32;
  let mut not_found = 0u32;

  for file in &export.files {
    let repo_file: PathBuf = ctx.repo_path.join(file);
    let home_file: PathBuf = ctx.home_path.join(file);

    if !repo_file.exists() {
      eprintln!("  Warning: '{}' not found in repository, skipping", file);
      not_found += 1;
      continue;
    }

    // Create parent directories in $HOME if needed
    if let Some(parent) = home_file.parent()
      && !parent.exists()
    {
      fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Check if symlink already exists and is correct
    if home_file
      .symlink_metadata()
      .map(|m: fs::Metadata| m.is_symlink())
      .unwrap_or(false)
      && let Ok(target) = fs::read_link(&home_file)
      && target == repo_file
    {
      skipped += 1;
      continue;
    }

    // Remove existing file/symlink if present
    if home_file.exists() || home_file.symlink_metadata().is_ok() {
      if home_file.is_dir() && !home_file.is_symlink() {
        fs::remove_dir_all(&home_file)?;
      } else {
        fs::remove_file(&home_file)?;
      }
    }

    create_symlink(&repo_file, &home_file)?;
    created += 1;
  }

  println!(
    "Imported: {} created, {} skipped (already correct), {} not found in repo",
    created, skipped, not_found
  );

  Ok(())
}

fn print_long(entries: &[ManagedEntry]) -> Result<()> {
  let max_len = entries
    .iter()
    .map(|e| e.relative.display().to_string().len())
    .max()
    .unwrap_or(0);

  println!("{:<width$}  TYPE  SIZE", "FILE", width = max_len + 2);

  for entry in entries {
    let metadata = fs::metadata(&entry.repo_path)?;

    let file_type = if metadata.is_dir() { "dir" } else { "file" };
    let size = if metadata.is_file() {
      format_size(metadata.len())
    } else {
      format_size(0)
    };

    println!(
      "{:<width$}  {:<4}  {}",
      entry.relative.display(),
      file_type,
      size,
      width = max_len + 2
    );
  }

  Ok(())
}

fn print_tree(root: &Path) -> Result<()> {
  fn walk(path: &Path, prefix: String) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?
      .filter_map(|e| e.ok())
      .filter(|e| e.file_name() != ".git")
      .collect();

    entries.sort_by_key(|e| e.file_name());

    let len = entries.len();

    for (i, entry) in entries.into_iter().enumerate() {
      let is_last = i == len - 1;
      let name = entry.file_name().to_string_lossy().to_string();
      let path = entry.path();

      let branch = if is_last { "└── " } else { "├── " };
      println!("{}{}{}", prefix, branch, name);

      if path.is_dir() {
        let new_prefix = if is_last {
          format!("{}    ", prefix)
        } else {
          format!("{}│   ", prefix)
        };
        walk(&path, new_prefix)?;
      }
    }

    Ok(())
  }

  println!("{}", root.file_name().unwrap_or_default().to_string_lossy());
  walk(root, String::new())?;

  Ok(())
}
