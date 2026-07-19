use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use console::style;
use serde::{Deserialize, Serialize};
use tildr_core::context::Context;
use tildr_repo::scatildr_repo;
use tildr_utils::{fs::tildr_dir, pager::page_string};
use walkdir::WalkDir;

use crate::utils::auto_commit::auto_commit;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Profiles {
  pub active: Option<String>,
  #[serde(default)]
  pub profiles: std::collections::HashMap<String, ProfileDef>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ProfileDef {
  pub description: Option<String>,
}

impl Profiles {
  fn path(ctx: &Context) -> PathBuf {
    tildr_dir(&ctx.repo_path).join("profiles.json")
  }

  pub fn load(ctx: &Context) -> Result<Self> {
    let path = Self::path(ctx);
    let mut profiles: Profiles = if !path.exists() {
      Self::default()
    } else {
      let data = fs::read_to_string(&path).context("Failed to read profiles file")?;
      serde_json::from_str(&data).context("Failed to parse profiles file")?
    };

    // Always ensure default profile exists
    if !profiles.profiles.contains_key("default") {
      profiles
        .profiles
        .insert("default".to_string(), ProfileDef::default());
    }

    Ok(profiles)
  }

  pub fn save(&self, ctx: &Context) -> Result<()> {
    let path = Self::path(ctx);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(self).context("Failed to serialize profiles")?;
    fs::write(&path, data).context("Failed to write profiles file")?;
    Ok(())
  }

  pub fn resolve(&self, repo_path: &Path, file: &str) -> PathBuf {
    if let Some(active) = self.active.as_deref().filter(|a| *a != "default") {
      let candidate = repo_path.join("profiles").join(active).join(file);
      if candidate.exists() {
        return candidate;
      }
    }
    repo_path.join("profiles/default").join(file)
  }
}

/// Returns the names of all profiles that have a physical copy of `file`,
/// sorted alphabetically.
pub fn variants_of(repo_path: &Path, file: &str, known_profiles: &[String]) -> Vec<String> {
  let mut out: Vec<String> = known_profiles
    .iter()
    .filter(|name| repo_path.join("profiles").join(name).join(file).exists())
    .cloned()
    .collect();
  out.sort();
  out
}

pub fn run(ctx: &Context, mode: &tildr_domain::ProfileMode) -> Result<()> {
  match mode {
    tildr_domain::ProfileMode::Create { name, description } => create(ctx, name, description),
    tildr_domain::ProfileMode::Add { files, from, to } => transfer(ctx, from, to, files, false),
    tildr_domain::ProfileMode::Mv { files, from, to } => transfer(ctx, from, to, files, true),
    tildr_domain::ProfileMode::Delete { name } => delete(ctx, name),
    tildr_domain::ProfileMode::Rename { from, to } => rename(ctx, from, to),
    tildr_domain::ProfileMode::List { long, less, name } => {
      list(ctx, *long, *less, name.as_deref())
    }
    tildr_domain::ProfileMode::Set { name } => set(ctx, name),
    tildr_domain::ProfileMode::Unset => unset(ctx),
    tildr_domain::ProfileMode::Current => current(ctx),
    tildr_domain::ProfileMode::Migrate { dry_run } => migrate(ctx, *dry_run),
  }
}

fn create(ctx: &Context, name: &str, description: &Option<String>) -> Result<()> {
  if name == "default" {
    anyhow::bail!("'default' is a reserved name and cannot be used for a profile.");
  }
  let mut profiles = Profiles::load(ctx)?;
  if profiles.profiles.contains_key(name) {
    anyhow::bail!("Profile '{}' already exists.", name);
  }
  let def = ProfileDef {
    description: description.clone(),
  };
  profiles.profiles.insert(name.to_string(), def);
  profiles.save(ctx)?;
  println!(
    "{} Profile '{}' created.",
    style("Created:").green().bold(),
    name
  );
  auto_commit(ctx, &format!("profile create {}", name));
  Ok(())
}

fn repo_entries(ctx: &Context) -> Result<Vec<tildr_repo::ManagedEntry>> {
  scatildr_repo(&ctx.repo_path)
}

fn resolve_files(ctx: &Context, from: &str, files: &[String]) -> Result<Vec<String>> {
  if files.is_empty() {
    let entries = repo_entries(ctx)?;
    if from == "default" {
      // In the filesystem model, all repo entries are tracked.
      // Return all files so the user can choose or transfer everything.
      let mut names: Vec<String> = entries
        .into_iter()
        .map(|e| e.relative.to_string_lossy().to_string())
        .collect();
      names.sort();
      names.dedup();
      Ok(names)
    } else {
      let mut names: Vec<String> = entries
        .into_iter()
        .filter(|e| e.profile == from)
        .map(|e| e.relative.to_string_lossy().to_string())
        .collect();
      names.sort();
      names.dedup();
      Ok(names)
    }
  } else {
    expand_files(ctx, files, from)
  }
}

fn expand_files(ctx: &Context, files: &[String], from_profile: &str) -> Result<Vec<String>> {
  let base = ctx.repo_path.join("profiles").join(from_profile);
  let mut result = Vec::new();
  for file in files {
    // First try relative to the profile directory
    let profile_path = base.join(file);
    let path = if profile_path.exists() {
      profile_path
    } else {
      // Fallback: try relative to repo root (user may have typed full path)
      ctx.repo_path.join(file)
    };
    if path.is_dir() {
      let strip_base = if path.starts_with(&base) {
        &base
      } else {
        &ctx.repo_path
      };
      for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
          let relative = entry
            .path()
            .strip_prefix(strip_base)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
          result.push(relative);
        }
      }
    } else if path.exists() {
      let relative = path
        .strip_prefix(&base)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
      result.push(relative);
    } else {
      result.push(file.clone());
    }
  }
  result.sort();
  result.dedup();
  Ok(result)
}

fn transfer(
  ctx: &Context,
  from: &str,
  to: &str,
  files: &[String],
  remove_source: bool,
) -> Result<()> {
  if from == to {
    println!("{}", style("Source and destination are the same.").dim());
    return Ok(());
  }

  let profiles = Profiles::load(ctx)?;

  if from != "default" && !profiles.profiles.contains_key(from) {
    anyhow::bail!("Profile '{}' not found.", from);
  }

  if to != "default" && !profiles.profiles.contains_key(to) {
    anyhow::bail!("Profile '{}' not found.", to);
  }

  let file_list = resolve_files(ctx, from, files)?;
  if file_list.is_empty() {
    println!("{}", style("Nothing to do.").dim());
    return Ok(());
  }

  let action = if remove_source { "Moved" } else { "Added" };
  let action_fn = |s: &str| -> String {
    if remove_source {
      style(s).cyan().to_string()
    } else {
      style(s).green().to_string()
    }
  };

  let from_profile = from != "default";
  let to_profile = to != "default";
  let mut count = 0;

  for file in &file_list {
    let src = if from_profile {
      ctx.repo_path.join("profiles").join(from).join(file)
    } else {
      ctx.repo_path.join(file)
    };

    if !src.exists() {
      println!("  {} {} (file not found)", style("Skipped:").yellow(), file);
      continue;
    }

    let dst = if to_profile {
      let dir = ctx.repo_path.join("profiles").join(to);
      if let Some(parent) = dir.join(file).parent() {
        fs::create_dir_all(parent)?;
      }
      dir.join(file)
    } else {
      ctx.repo_path.join(file)
    };

    fs::copy(&src, &dst)?;

    if remove_source {
      fs::remove_file(&src)?;
    }

    count += 1;
    let direction = if from_profile && to_profile {
      format!("profiles/{from}/{file} -> profiles/{to}/{file}")
    } else if from_profile {
      format!("profiles/{from}/{file} -> {file}")
    } else {
      format!("{file} -> profiles/{to}/{file}")
    };
    println!("  {} {}", action_fn(action), direction);
  }

  if remove_source && from_profile {
    // Clean up empty profile directory
    let dir = ctx.repo_path.join("profiles").join(from);
    if dir.exists() && dir.read_dir()?.next().is_none() {
      fs::remove_dir(&dir)?;
    }
  }

  auto_commit(
    ctx,
    &format!(
      "profile {} {} -> {} ({count})",
      if remove_source { "mv" } else { "add" },
      from,
      to
    ),
  );
  Ok(())
}

fn delete(ctx: &Context, name: &str) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  let _def = profiles
    .profiles
    .remove(name)
    .context(format!("Profile '{name}' not found."))?;

  let profile_dir = ctx.repo_path.join("profiles").join(name);
  let default_dir = ctx.repo_path.join("profiles").join("default");

  // Restore files to profiles/default/ if they don't exist there
  if profile_dir.exists() {
    for entry in WalkDir::new(&profile_dir)
      .into_iter()
      .filter_map(|e| e.ok())
    {
      if entry.file_type().is_file() {
        let relative = entry
          .path()
          .strip_prefix(&profile_dir)
          .unwrap_or(entry.path());
        let target = default_dir.join(relative);
        if !target.exists() {
          if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
          }
          fs::copy(entry.path(), &target)?;
          println!("  {} {}", style("Restored:").green(), relative.display());
        }
      }
    }
  }

  if profiles.active.as_deref() == Some(name) {
    profiles.active = None;
  }

  if profile_dir.exists() {
    fs::remove_dir_all(&profile_dir)?;
  }

  profiles.save(ctx)?;
  println!(
    "{} Profile '{name}' deleted.",
    style("Deleted:").red().bold()
  );
  auto_commit(ctx, &format!("profile delete {name}"));
  Ok(())
}

fn rename(ctx: &Context, from: &str, to: &str) -> Result<()> {
  if from == to {
    println!("{}", style("Source and destination are the same.").dim());
    return Ok(());
  }

  let mut profiles = Profiles::load(ctx)?;

  if !profiles.profiles.contains_key(from) {
    anyhow::bail!("Profile '{from}' not found.");
  }

  if profiles.profiles.contains_key(to) {
    anyhow::bail!("Profile '{to}' already exists.");
  }

  if from == "default" || to == "default" {
    anyhow::bail!("'default' is a reserved name and cannot be renamed.");
  }

  let old_dir = ctx.repo_path.join("profiles").join(from);
  let new_dir = ctx.repo_path.join("profiles").join(to);

  // Rename the profile directory
  fs::rename(&old_dir, &new_dir)?;

  // Move the profile definition
  let def = profiles
    .profiles
    .remove(from)
    .context("Profile not found")?;
  profiles.profiles.insert(to.to_string(), def);

  // If the renamed profile was active, update the active profile
  if profiles.active.as_deref() == Some(from) {
    profiles.active = Some(to.to_string());
  }

  profiles.save(ctx)?;

  println!(
    "{} Profile '{from}' renamed to '{to}'.",
    style("Renamed:").green().bold()
  );
  auto_commit(ctx, &format!("profile rename {from} {to}"));
  Ok(())
}

fn list(ctx: &Context, long: bool, less: bool, name: Option<&str>) -> Result<()> {
  let profiles = Profiles::load(ctx)?;
  let known_profiles: Vec<String> = profiles.profiles.keys().cloned().collect();

  let names: Vec<String> = if let Some(name) = name {
    if !profiles.profiles.contains_key(name) {
      anyhow::bail!("Profile '{name}' not found.");
    }
    vec![name.to_string()]
  } else {
    if profiles.profiles.is_empty() {
      println!("{}", style("No profiles defined.").dim());
      return Ok(());
    }
    let mut names: Vec<String> = profiles.profiles.keys().cloned().collect();
    names.sort();
    names
  };

  let mut buf = String::new();

  for name in &names {
    let def = &profiles.profiles[name.as_str()];
    let marker = if profiles.active.as_deref() == Some(name.as_str()) {
      style(" (active)").green().bold().to_string()
    } else {
      String::new()
    };
    let desc = def.description.as_deref().unwrap_or("no description");

    // Scan profile directory on disk to get file count
    let profile_dir = ctx.repo_path.join("profiles").join(name);
    let file_count = if profile_dir.exists() {
      WalkDir::new(&profile_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count()
    } else {
      0
    };

    if long {
      writeln!(buf, "  {marker}{} — {desc}", style(name).cyan())?;
      if file_count == 0 {
        writeln!(buf, "    (no files)")?;
      } else {
        for entry in WalkDir::new(&profile_dir)
          .into_iter()
          .filter_map(|e| e.ok())
          .filter(|e| e.file_type().is_file())
        {
          let relative = entry
            .path()
            .strip_prefix(&profile_dir)
            .unwrap_or(entry.path());
          writeln!(buf, "    {}", relative.display())?;
        }
      }
    } else {
      writeln!(
        buf,
        "  {marker}{} — {desc} [{file_count} file(s)]",
        style(name).cyan()
      )?;
    }

    // Show variants in long mode
    if long && file_count > 0 {
      for entry in WalkDir::new(&profile_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
      {
        let relative = entry
          .path()
          .strip_prefix(&profile_dir)
          .unwrap_or(entry.path());
        let file_str = relative.to_string_lossy();
        let variants = variants_of(&ctx.repo_path, &file_str, &known_profiles);
        if variants.len() > 1 {
          writeln!(
            buf,
            "    {} variants: {}",
            style(&*file_str).dim(),
            variants.join(", ")
          )?;
        }
      }
    }
  }

  if less {
    page_string(&buf)?;
  } else {
    print!("{buf}");
  }

  Ok(())
}

fn set(ctx: &Context, name: &str) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  if !profiles.profiles.contains_key(name) {
    anyhow::bail!("Profile '{name}' not found.");
  }
  profiles.active = Some(name.to_string());
  profiles.save(ctx)?;
  println!(
    "{} Profile '{name}' activated.",
    style("Set:").green().bold()
  );
  auto_commit(ctx, &format!("profile set {name}"));
  Ok(())
}

fn unset(ctx: &Context) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  if profiles.active.is_none() {
    println!("{}", style("No profile is currently active.").dim());
    return Ok(());
  }
  let old = profiles.active.take();
  profiles.save(ctx)?;
  println!(
    "{} Profile '{}' deactivated.",
    style("Unset:").yellow().bold(),
    old.unwrap_or_default()
  );
  auto_commit(ctx, "profile unset");
  Ok(())
}

fn current(ctx: &Context) -> Result<()> {
  let profiles = Profiles::load(ctx)?;
  match &profiles.active {
    Some(name) => println!("Active profile: {}", style(name).cyan()),
    None => println!("No profile is currently active. Using default files."),
  }
  Ok(())
}

fn migrate(ctx: &Context, dry_run: bool) -> Result<()> {
  // Find files at repo root that should be in profiles/default/
  let root_entries: Vec<_> = fs::read_dir(&ctx.repo_path)?
    .filter_map(|e| e.ok())
    .filter(|e| {
      let name = e.file_name().to_string_lossy().to_string();
      !name.starts_with('.') && name != "profiles"
    })
    .collect();

  if root_entries.is_empty() {
    println!("{}", style("Nothing to migrate.").dim());
    return Ok(());
  }

  let default_dir = ctx.repo_path.join("profiles").join("default");
  fs::create_dir_all(&default_dir)?;

  let mut count = 0;

  for entry in &root_entries {
    let name = entry.file_name().to_string_lossy().to_string();

    // Skip if already exists in profiles/default/
    let target = default_dir.join(&name);
    if target.exists() {
      println!(
        "  {} {} (already in profiles/default/)",
        style("Skipped:").dim(),
        name
      );
      continue;
    }

    if dry_run {
      println!(
        "  {} {} -> profiles/default/{name}",
        style("Would migrate:").cyan(),
        name
      );
    } else {
      let source = entry.path();
      fs::rename(&source, &target)?;
      println!(
        "  {} {} -> profiles/default/{name}",
        style("Migrated:").green(),
        name
      );
    }
    count += 1;
  }

  if count == 0 {
    println!("{}", style("Nothing to migrate.").dim());
    return Ok(());
  }

  if !dry_run {
    auto_commit(
      ctx,
      &format!("migrate {count} file(s) to profiles/default/"),
    );
  }

  println!(
    "\n{} {count} file(s) migrated to profiles/default/{}",
    if dry_run {
      style("Would migrate:").cyan()
    } else {
      style("Migrated:").green().bold()
    },
    if dry_run { " (dry run)" } else { "" }
  );

  Ok(())
}
