use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use console::style;
use serde::{Deserialize, Serialize};
use tildr_core::context::Context;
use tildr_utils::fs::tildr_dir;

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

pub fn run(ctx: &Context, mode: &tildr_domain::GroupMode) -> Result<()> {
  match mode {
    tildr_domain::GroupMode::Create { name, files } => create(ctx, name, files),
    tildr_domain::GroupMode::Add { name, files } => add(ctx, name, files),
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
  groups.groups.insert(name.to_string(), files.to_vec());
  groups.save(ctx)?;
  println!(
    "{} Group '{}' created with {} file(s).",
    style("Created:").green().bold(),
    name,
    files.len()
  );
  Ok(())
}

fn add(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  let group = groups
    .groups
    .entry(name.to_string())
    .or_insert_with(Vec::new);
  let before = group.len();
  for file in files {
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
  Ok(())
}

fn remove(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut groups = Groups::load(ctx)?;
  let group = groups
    .groups
    .get_mut(name)
    .context(format!("Group '{}' not found.", name))?;
  let before = group.len();
  group.retain(|f| !files.contains(f));
  let removed = before - group.len();
  groups.save(ctx)?;
  println!(
    "{} {} file(s) removed from group '{}'.",
    style("Updated:").green().bold(),
    removed,
    name
  );
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

  for file in files {
    let src = ctx.repo_path.join(file);
    let dst = home.join(file);
    if !src.exists() {
      println!(
        "{} {} (source not in repo)",
        style("Skipped:").yellow(),
        file
      );
      continue;
    }
    if dst.exists() && !dst.is_symlink() {
      println!(
        "{} {} (not a symlink, use --force?)",
        style("Skipped:").yellow(),
        file
      );
      continue;
    }
    tildr_fs::symlink::create_symlink(&src, &dst)
      .context(format!("Failed to symlink '{}'", file))?;
    println!("{} {}", style("Linked:").green(), file);
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
