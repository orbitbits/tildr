use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use console::style;
use dialoguer::Input;
use serde::{Deserialize, Serialize};
use tildr_core::context::Context;
use tildr_fs::symlink::{create_symlink, is_symlink, is_symlink_to};
use tildr_ui::prompt::MinimalTheme;
use tildr_utils::{fs::tildr_dir, sys::has_display};

use crate::profile::Profiles;
use crate::utils::auto_commit::auto_commit;
use crate::utils::target::{
  ResolvedTarget, logical_home_candidates, resolve_targets, storage_to_home_relative,
};

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
    let mut groups: Groups = serde_json::from_str(&data).context("Failed to parse groups file")?;
    for files in groups.groups.values_mut() {
      let mut normalized = BTreeSet::new();
      for file in files.iter() {
        normalized.insert(normalize_stored_group_file(ctx, file));
      }
      *files = normalized.into_iter().collect();
    }
    Ok(groups)
  }

  fn save(&self, ctx: &Context) -> Result<()> {
    let path = Self::path(ctx);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).context("Failed to create groups directory")?;
    }
    let data = serde_json::to_string_pretty(self).context("Failed to serialize groups")?;
    fs::write(&path, data).context("Failed to write groups file")?;
    Ok(())
  }
}

fn normalize_stored_group_file(ctx: &Context, file: &str) -> String {
  let logical = storage_to_home_relative(&ctx.repo_path, std::path::Path::new(file));
  let logical_str = logical.display().to_string();
  let path = std::path::Path::new(&logical_str);

  if path.is_absolute()
    || logical_str == "~"
    || logical_str.starts_with("~/")
    || logical_str == "$HOME"
    || logical_str.starts_with("$HOME/")
  {
    let home_path = tildr_fs::paths::resolve_home_path(&logical_str, &ctx.home_path);
    if let Ok(relative) = home_path.strip_prefix(&ctx.home_path) {
      return relative.display().to_string();
    }
  }

  logical_str
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
          files.push(
            storage_to_home_relative(&ctx.repo_path, std::path::Path::new(&relative))
              .display()
              .to_string(),
          );
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
    tildr_domain::GroupMode::Rename { from, to } => rename(ctx, from.as_deref(), to.as_deref()),
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
  let normalized = resolve_group_files(ctx, files)?;
  groups.groups.insert(name.to_string(), normalized);
  groups.save(ctx)?;
  println!(
    "{} Group '{}' created with {} file(s).",
    style("Created:").green().bold(),
    name,
    groups.groups[name].len()
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

  let resolved_files = resolve_group_files(ctx, &raw_files)?;

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
  let normalized = normalize_group_patterns(ctx, files)?;
  let mut groups = Groups::load(ctx)?;
  let group = groups
    .groups
    .get_mut(name)
    .context(format!("Group '{}' not found.", name))?;
  let before = group.len();
  group.retain(|f| {
    !normalized
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
  auto_commit(ctx, &format!("group delete {name}"));
  Ok(())
}

fn rename(ctx: &Context, from: Option<&str>, to: Option<&str>) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  let from = prompt_required("Enter current group name", from)?;
  let to = prompt_required("Enter new group name", to)?;

  if from == to {
    println!("{}", style("Source and destination are the same.").dim());
    return Ok(());
  }

  let files = groups
    .groups
    .remove(&from)
    .context(format!("Group '{from}' not found."))?;

  if groups.groups.contains_key(&to) {
    groups.groups.insert(from.clone(), files);
    anyhow::bail!("Group '{to}' already exists.");
  }

  groups.groups.insert(to.clone(), files);
  groups.save(ctx)?;
  println!(
    "{} Group '{from}' renamed to '{to}'.",
    style("Renamed:").green().bold()
  );
  auto_commit(ctx, &format!("group rename {from} {to}"));
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

fn prompt_required(prompt: &str, value: Option<&str>) -> Result<String> {
  if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
    return Ok(value.trim().to_string());
  }

  loop {
    let input: String = Input::with_theme(&MinimalTheme)
      .with_prompt(prompt)
      .interact_text()?;
    let input = input.trim();
    if !input.is_empty() {
      return Ok(input.to_string());
    }
    println!("{}", style("Value cannot be empty.").yellow());
  }
}

fn apply(ctx: &Context, name: &str) -> Result<()> {
  let groups = Groups::load(ctx)?;
  let files = groups
    .groups
    .get(name)
    .context(format!("Group '{}' not found.", name))?;

  let profiles = Profiles::load(ctx)?;

  let mut linked = 0;
  let mut up_to_date = 0;
  let mut skipped = 0;

  for file in files {
    let src = profiles.resolve(&ctx.repo_path, file);
    let dst = ctx.home_path.join(file);

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

  for file in files {
    let dst = ctx.home_path.join(file);
    if dst.is_symlink() {
      std::fs::remove_file(&dst).context(format!("Failed to remove symlink '{}'", file))?;
      println!("{} {}", style("Unlinked:").red(), file);
    } else {
      println!("{} {} (not a symlink)", style("Skipped:").yellow(), file);
    }
  }
  Ok(())
}

fn resolve_group_files(ctx: &Context, files: &[String]) -> Result<Vec<String>> {
  let resolved = resolve_targets(ctx, files, None)?;
  let mut logical_files = BTreeSet::new();

  for target in resolved {
    match target {
      ResolvedTarget::Interactive => {}
      ResolvedTarget::File(entry) => {
        if entry.repo_path.is_dir() {
          insert_dir_files(ctx, &entry.repo_path, &mut logical_files);
        } else {
          logical_files.insert(entry.relative.display().to_string());
        }
      }
      ResolvedTarget::Dir { entries, .. } => {
        for entry in entries {
          logical_files.insert(entry.relative.display().to_string());
        }
      }
    }
  }

  Ok(logical_files.into_iter().collect())
}

fn insert_dir_files(
  ctx: &Context,
  repo_dir: &std::path::Path,
  logical_files: &mut BTreeSet<String>,
) {
  for entry in walkdir::WalkDir::new(repo_dir)
    .into_iter()
    .filter_map(|entry| entry.ok())
  {
    if entry.file_type().is_file() {
      logical_files.insert(
        storage_to_home_relative(&ctx.repo_path, entry.path())
          .display()
          .to_string(),
      );
    }
  }
}

fn normalize_group_patterns(ctx: &Context, files: &[String]) -> Result<Vec<String>> {
  let mut patterns = BTreeSet::new();

  for file in files {
    let logical = storage_to_home_relative(&ctx.repo_path, std::path::Path::new(file));
    for candidate in logical_home_candidates(ctx, &logical.display().to_string())? {
      patterns.insert(candidate.display().to_string());
    }
  }

  Ok(patterns.into_iter().collect())
}
