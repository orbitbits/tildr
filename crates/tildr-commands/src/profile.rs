use std::collections::{HashMap, HashSet};
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
  pub profiles: HashMap<String, ProfileDef>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ProfileDef {
  pub description: Option<String>,
  pub files: HashMap<String, String>,
}

impl Profiles {
  fn path(ctx: &Context) -> PathBuf {
    tildr_dir(&ctx.repo_path).join("profiles.json")
  }

  pub fn load(ctx: &Context) -> Result<Self> {
    let path = Self::path(ctx);
    if !path.exists() {
      return Ok(Self::default());
    }
    let data = fs::read_to_string(&path).context("Failed to read profiles file")?;
    let profiles: Profiles =
      serde_json::from_str(&data).context("Failed to parse profiles file")?;
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
    if let Some(ref active) = self.active
      && let Some(profile) = self.profiles.get(active)
      && let Some(variant) = profile.files.get(file)
    {
      return repo_path.join(variant);
    }
    repo_path.join(file)
  }
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
    files: HashMap::new(),
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

fn repo_entries_set(ctx: &Context) -> Result<HashSet<String>> {
  let entries = scatildr_repo(&ctx.repo_path)?;
  Ok(
    entries
      .into_iter()
      .map(|e| e.relative.to_string_lossy().to_string())
      .collect(),
  )
}

fn all_tracked_files(profiles: &Profiles) -> HashSet<String> {
  profiles
    .profiles
    .values()
    .flat_map(|def| def.files.keys().cloned())
    .collect()
}

fn resolve_files(
  ctx: &Context,
  profiles: &Profiles,
  from: &str,
  files: &[String],
) -> Result<Vec<String>> {
  if files.is_empty() {
    if from == "default" {
      let all = repo_entries_set(ctx)?;
      let tracked = all_tracked_files(profiles);
      let mut orphans: Vec<String> = all.difference(&tracked).cloned().collect();
      orphans.sort();
      Ok(orphans)
    } else {
      let def = profiles
        .profiles
        .get(from)
        .context(format!("Profile '{}' not found.", from))?;
      let mut keys: Vec<String> = def.files.keys().cloned().collect();
      keys.sort();
      Ok(keys)
    }
  } else {
    expand_files(ctx, files)
  }
}

fn expand_files(ctx: &Context, files: &[String]) -> Result<Vec<String>> {
  let mut result = Vec::new();
  for file in files {
    let path = ctx.repo_path.join(file);
    if path.is_dir() {
      for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
          let relative = entry
            .path()
            .strip_prefix(&ctx.repo_path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
          result.push(relative);
        }
      }
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

  let mut profiles = Profiles::load(ctx)?;

  if from != "default" && !profiles.profiles.contains_key(from) {
    anyhow::bail!("Profile '{}' not found.", from);
  }

  if to != "default" && !profiles.profiles.contains_key(to) {
    anyhow::bail!("Profile '{}' not found.", to);
  }

  let file_list = resolve_files(ctx, &profiles, from, files)?;
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

    if from_profile
      && !profiles
        .profiles
        .get(from)
        .is_some_and(|d| d.files.contains_key(file))
    {
      println!(
        "  {} {} (not tracked in profile '{from}')",
        style("Skipped:").dim(),
        file
      );
      continue;
    }

    if !src.exists() {
      println!("  {} {} (file not found)", style("Skipped:").yellow(), file);
      if remove_source && from_profile {
        profiles
          .profiles
          .get_mut(from)
          .context(format!("Profile '{from}' not found"))?
          .files
          .remove(file);
      }
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

    if to_profile {
      profiles
        .profiles
        .get_mut(to)
        .context(format!("Profile '{to}' not found"))?
        .files
        .insert(file.clone(), format!("profiles/{to}/{file}"));
    }

    if from_profile {
      profiles
        .profiles
        .get_mut(from)
        .context(format!("Profile '{from}' not found"))?
        .files
        .remove(file);
    }

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
    let dir = ctx.repo_path.join("profiles").join(from);
    if dir.exists() {
      fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).ok();
  }

  profiles.save(ctx)?;
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
  let def = profiles
    .profiles
    .remove(name)
    .context(format!("Profile '{name}' not found."))?;

  let profile_dir = ctx.repo_path.join("profiles").join(name);

  for file in def.files.keys() {
    let target = ctx.repo_path.join(file);
    if !target.exists() {
      let src = profile_dir.join(file);
      if src.exists() {
        if let Some(parent) = target.parent() {
          fs::create_dir_all(parent)?;
        }
        fs::copy(&src, &target)?;
        println!("  {} {}", style("Restored:").green(), file);
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

  // Get the profile definition and update its file paths
  let mut def = profiles.profiles.remove(from).unwrap();
  for file in def.files.values_mut() {
    *file = file.replace(&format!("profiles/{from}/"), &format!("profiles/{to}/"));
  }

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

  let names: Vec<&String> = if let Some(name) = name {
    if !profiles.profiles.contains_key(name) {
      anyhow::bail!("Profile '{name}' not found.");
    }
    vec![profiles.profiles.get_key_value(name).unwrap().0]
  } else {
    if profiles.profiles.is_empty() {
      println!("{}", style("No profiles defined.").dim());
      return Ok(());
    }
    let mut names: Vec<&String> = profiles.profiles.keys().collect();
    names.sort();
    names
  };

  let mut buf = String::new();

  for name in names {
    let def = &profiles.profiles[name.as_str()];
    let marker = if profiles.active.as_deref() == Some(name.as_str()) {
      style(" (active)").green().bold().to_string()
    } else {
      String::new()
    };
    let desc = def.description.as_deref().unwrap_or("no description");

    if long {
      writeln!(buf, "  {marker}{} — {desc}", style(name).cyan())?;
      if def.files.is_empty() {
        writeln!(buf, "    (no files)")?;
      } else {
        let mut file_list: Vec<&String> = def.files.keys().collect();
        file_list.sort();
        for file in file_list {
          writeln!(buf, "    {file}")?;
        }
      }
    } else {
      let count = def.files.len();
      writeln!(
        buf,
        "  {marker}{} — {desc} [{count} file(s)]",
        style(name).cyan()
      )?;
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
