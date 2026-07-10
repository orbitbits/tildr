use anyhow::Result;
use std::{fs, path::Path};
use tildr_core::context::Context;
use tildr_repo::ManagedEntry;
use tildr_ui::info;
use tildr_utils::fs::format_size;

pub struct ListArgs {
  pub tree: bool,
  pub long: bool,
}

pub fn run(ctx: &Context, args: ListArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    info("Repository not initialized. Run `tildr init` first.");
    return Ok(());
  }

  if args.tree {
    print_tree(&ctx.repo_path)?;
    return Ok(());
  }

  // let entries = collect_entries(&ctx.repo_path)?;
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

// fn collect_entries(root: &Path) -> Result<Vec<PathBuf>> {
//   let entries = WalkDir::new(root)
//     .min_depth(1)
//     .into_iter()
//     .filter_map(|e| e.ok())
//     .filter(|e| !e.path().components().any(|c| c.as_os_str() == ".git"))
//     .filter(|e| e.file_type().is_file())
//     .map(|e| e.path().strip_prefix(root).unwrap().to_path_buf())
//     .collect();

//   Ok(entries)
// }

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
