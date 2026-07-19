use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use console::style;
use serde::{Deserialize, Serialize};
use tildr_core::context::Context;
use tildr_fs::symlink::{create_symlink, is_symlink, is_symlink_to};
use tildr_utils::{fs::tildr_dir, sys::has_display};

use crate::profile::Profiles;
use crate::utils::auto_commit::auto_commit;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Groups {
  pub groups: HashMap<String, Vec<String>>,
}

impl Groups {
  fn path(ctx: &Context) -> PathBuf {
    tildr_dir(&ctx.repo_path).join("groups.json")
  }

  fn load(ctx: &Context) -> Result<Self> {
    let path = Self::path(ctx);
    if !path.exists() {
      return Ok(Self::default());
    }
    let data = fs::read_to_string(&path).context("Failed to read groups file")?;
    let groups: Groups = serde_json::from_str(&data).context("Failed to parse groups file")?;
    Ok(groups)
  }

  fn save(&self, ctx: &Context) -> Result<()> {
    let path = Self::path(ctx);
    let data = serde_json::to_string_pretty(self).context("Failed to serialize groups")?;
    fs::write(&path, data).context("Failed to write groups file")?;
    Ok(())
  }
}

/// Strip `profiles/<name>/` prefix from a repo-relative path to get the HOME-relative path.
fn strip_profile_prefix(_repo_path: &std::path::Path, full: &str) -> String {
  if let Some(stripped) = full.strip_prefix("profiles/")
    && let Some(rest) = stripped.find('/')
  {
    let after = &stripped[rest + 1..];
    if !after.is_empty() {
      return after.to_string();
    }
  }
  full.to_string()
}

fn pick_files_for_group(ctx: &Context) -> Result<Vec<String>> {
  if has_display() {
    let picked = rfd::FileDialog::new()
      .set_directory(&ctx.repo_path)
      .pick_files();

    match picked {
      Some(paths) => {
        let mut files = Vec::new();
        for path in paths {
          let relative = path
            .strip_prefix(&ctx.repo_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
          // Convert repo-relative path to HOME-relative path
          let home_relative = strip_profile_prefix(&ctx.repo_path, &relative);
          files.push(home_relative);
        }
        Ok(files)
      }
      None => Ok(Vec::new()),
    }
  } else {
    use tildr_ui::prompt::MinimalTheme;
    let input: String = dialoguer::Input::with_theme(&MinimalTheme)
      .with_prompt("File path (relative to HOME)")
      .allow_empty(true)
      .interact_text()?;
    if input.is_empty() {
      Ok(Vec::new())
    } else {
      Ok(input.split_whitespace().map(|s| s.to_string()).collect())
    }
  }
}

pub fn run(ctx: &Context, mode: &tildr_domain::GroupMode) -> Result<()> {
  match mode {
    tildr_domain::GroupMode::Create { name, files } => create(ctx, name, files),
    tildr_domain::GroupMode::Add { name, files } => add(ctx, name, files.as_deref()),
    tildr_domain::GroupMode::Remove { name, files } => remove(ctx, name, files),
    tildr_domain::GroupMode::Delete { name } => delete(ctx, name),
    tildr_domain::GroupMode::List => list(ctx),
    tildr_domain::GroupMode::Apply { name } => apply(ctx, name),
    tildr_domain::GroupMode::Unlink { name } => unlink(ctx, name),
  }
}

fn create(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  if groups.groups.contains_key(name) {
    anyhow::bail!(
      "Group '{}' already exists. Use `tildr group add` instead.",
      name
    );
  }
  // Normalize all paths to HOME-relative
  let normalized: Vec<String> = files
    .iter()
    .map(|f| strip_profile_prefix(&ctx.repo_path, f))
    .collect();
  groups.groups.insert(name.to_string(), normalized);
  groups.save(ctx)?;
  println!(
    "{} Group '{}' created with {} file(s).",
    style("Created:").green().bold(),
    name,
    files.len()
  );
  auto_commit(ctx, &format!("group create {}", name));
  Ok(())
}

fn add(ctx: &Context, name: &str, files: Option<&[String]>) -> Result<()> {
  let raw_files = match files {
    Some(f) if !f.is_empty() => f.to_vec(),
    _ => pick_files_for_group(ctx)?,
  };

  if raw_files.is_empty() {
    println!("{}", style("No files selected.").dim());
    return Ok(());
  }

  let profiles = Profiles::load(ctx)?;
  let mut resolved_files = Vec::new();
  for file in &raw_files {
    // Normalize to HOME-relative path
    let home_relative = strip_profile_prefix(&ctx.repo_path, file);
    let repo_file = profiles.resolve(&ctx.repo_path, &home_relative);

    if repo_file.is_dir() {
      for entry in walkdir::WalkDir::new(&repo_file)
        .into_iter()
        .filter_map(|e| e.ok())
      {
        if entry.file_type().is_file() {
          let relative = entry
            .path()
            .strip_prefix(&ctx.repo_path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
          let home_rel = strip_profile_prefix(&ctx.repo_path, &relative);
          resolved_files.push(home_rel);
        }
      }
    } else {
      resolved_files.push(home_relative);
    }
  }

  let mut groups = Groups::load(ctx)?;
  let group = groups
    .groups
    .entry(name.to_string())
    .or_insert_with(Vec::new);
  let before = group.len();
  for file in &resolved_files {
    if !group.contains(file) {
      group.push(file.clone());
    }
  }
  let added = group.len() - before;
  groups.save(ctx)?;
  println!(
    "{} {} file(s) added to group '{}'.",
    style("Updated:").green().bold(),
    added,
    name
  );
  auto_commit(ctx, &format!("group add {}", name));
  Ok(())
}

fn remove(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  let group = groups
    .groups
    .get_mut(name)
    .context(format!("Group '{}' not found.", name))?;
  let before = group.len();
  group.retain(|f| {
    !files
      .iter()
      .any(|pattern| f == pattern || f.starts_with(&format!("{pattern}/")))
  });
  let removed = before - group.len();
  groups.save(ctx)?;
  println!(
    "{} {} file(s) removed from group '{}'.",
    style("Updated:").green().bold(),
    removed,
    name
  );
  auto_commit(ctx, &format!("group remove {}", name));
  Ok(())
}

fn delete(ctx: &Context, name: &str) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  groups
    .groups
    .remove(name)
    .context(format!("Group '{}' not found.", name))?;
  groups.save(ctx)?;
  println!(
    "{} Group '{}' deleted.",
    style("Deleted:").red().bold(),
    name
  );
  Ok(())
}

fn list(ctx: &Context) -> Result<()> {
  let groups = Groups::load(ctx)?;
  if groups.groups.is_empty() {
    println!("{}", style("No groups defined.").dim());
    println!(
      "{}",
      style("Create one with: tildr group create <name> --files <paths...>").dim()
    );
    return Ok(());
  }

  let mut sorted: Vec<_> = groups.groups.iter().collect();
  sorted.sort_by_key(|(name, _)| name.to_string());

  for (name, files) in &sorted {
    println!("{} ({})", style(name).green().bold(), files.len());
    for file in *files {
      println!("  {}", style(file).cyan());
    }
  }
  Ok(())
}

fn apply(ctx: &Context, name: &str) -> Result<()> {
  let groups = Groups::load(ctx)?;
  let files = groups
    .groups
    .get(name)
    .context(format!("Group '{}' not found.", name))?;

  let home = dirs::home_dir().context("Could not determine home directory")?;
  let profiles = Profiles::load(ctx)?;

  let mut linked = 0;
  let mut up_to_date = 0;
  let mut skipped = 0;

  for file in files {
    let src = profiles.resolve(&ctx.repo_path, file);
    let dst = home.join(file);

    if !src.exists() {
      println!(
        "{} {} (source not in repo)",
        style("Skipped:").yellow(),
        file
      );
      skipped += 1;
      continue;
    }

    // Correct symlink already in place
    if is_symlink(&dst) && is_symlink_to(&dst, &src) {
      up_to_date += 1;
      continue;
    }

    // Existing symlink points to wrong target → fix it
    if is_symlink(&dst) {
      fs::remove_file(&dst)?;
    } else if dst.exists() {
      println!(
        "{} {} (not a symlink, use --force?)",
        style("Skipped:").yellow(),
        file
      );
      skipped += 1;
      continue;
    }

    if let Some(parent) = dst.parent() {
      fs::create_dir_all(parent)?;
    }

    create_symlink(&src, &dst)?;
    println!("{} {}", style("Linked:").green(), file);
    linked += 1;
  }

  if !files.is_empty() && linked == 0 && skipped == 0 {
    println!("{}", style("Nothing to do.").dim());
  } else if !files.is_empty() {
    println!(
      "{} linked, {} up-to-date, {} skipped",
      style(linked).green(),
      style(up_to_date).dim(),
      style(skipped).yellow()
    );
  }

  Ok(())
}

fn unlink(ctx: &Context, name: &str) -> Result<()> {
  let groups = Groups::load(ctx)?;
  let files = groups
    .groups
    .get(name)
    .context(format!("Group '{}' not found.", name))?;

  let home = dirs::home_dir().context("Could not determine home directory")?;

  for file in files {
    let dst = home.join(file);
    if dst.is_symlink() {
      std::fs::remove_file(&dst).context(format!("Failed to remove symlink '{}'", file))?;
      println!("{} {}", style("Unlinked:").red(), file);
    } else {
      println!("{} {} (not a symlink)", style("Skipped:").yellow(), file);
    }
  }
  Ok(())
}
