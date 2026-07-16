use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use console::style;
use serde::{Deserialize, Serialize};
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_git::GitIntegration;
use tildr_utils::fs::tildr_dir;
use walkdir::WalkDir;

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
    tildr_domain::ProfileMode::Add { name, files } => add(ctx, name, files),
    tildr_domain::ProfileMode::Remove { name, files } => remove(ctx, name, files),
    tildr_domain::ProfileMode::List => list(ctx),
    tildr_domain::ProfileMode::Set { name } => set(ctx, name),
    tildr_domain::ProfileMode::Unset => unset(ctx),
    tildr_domain::ProfileMode::Current => current(ctx),
  }
}

fn create(ctx: &Context, name: &str, description: &Option<String>) -> Result<()> {
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

fn add(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  if !profiles.profiles.contains_key(name) {
    anyhow::bail!("Profile '{}' not found.", name);
  }

  let profile_dir = ctx.repo_path.join("profiles").join(name);
  fs::create_dir_all(&profile_dir)?;

  let mut raw_files = Vec::new();
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
          raw_files.push(relative);
        }
      }
    } else {
      raw_files.push(file.clone());
    }
  }

  let def = profiles.profiles.get_mut(name).unwrap();
  let mut added = 0;
  for file in &raw_files {
    let src = ctx.repo_path.join(file);
    if !src.exists() {
      println!(
        "{} {} (not found in repo)",
        style("Skipped:").yellow(),
        file
      );
      continue;
    }
    let dst = profile_dir.join(file);
    if let Some(parent) = dst.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::copy(&src, &dst)?;
    def
      .files
      .insert(file.clone(), format!("profiles/{}/{}", name, file));
    added += 1;
    println!(
      "{} {} -> profiles/{}/{}",
      style("Added:").green(),
      file,
      name,
      file
    );
  }

  profiles.save(ctx)?;
  auto_commit(ctx, &format!("profile add {} ({})", name, added));
  Ok(())
}

fn remove(ctx: &Context, name: &str, files: &[String]) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  let def = profiles
    .profiles
    .get_mut(name)
    .context(format!("Profile '{}' not found.", name))?;

  let profile_dir = ctx.repo_path.join("profiles").join(name);
  let mut removed = 0;
  for file in files {
    if def.files.remove(file).is_some() {
      let variant = profile_dir.join(file);
      if variant.exists() {
        fs::remove_file(&variant)?;
      }
      println!(
        "{} {} from profile '{}'",
        style("Removed:").yellow().bold(),
        file,
        name
      );
      removed += 1;
    } else {
      println!(
        "{} {} not found in profile '{}'",
        style("Skipped:").dim(),
        file,
        name
      );
    }
  }

  profiles.save(ctx)?;
  auto_commit(ctx, &format!("profile remove {} ({})", name, removed));
  Ok(())
}

fn list(ctx: &Context) -> Result<()> {
  let profiles = Profiles::load(ctx)?;
  if profiles.profiles.is_empty() {
    println!("{}", style("No profiles defined.").dim());
    return Ok(());
  }

  let mut names: Vec<&String> = profiles.profiles.keys().collect();
  names.sort();

  for name in names {
    let def = &profiles.profiles[name];
    let marker = if profiles.active.as_deref() == Some(name.as_str()) {
      style(" (active)").green().bold().to_string()
    } else {
      String::new()
    };
    let desc = def.description.as_deref().unwrap_or("no description");
    let count = def.files.len();
    println!(
      "  {} {} — {} [{} file(s)]",
      marker,
      style(name).cyan(),
      desc,
      count
    );
  }
  Ok(())
}

fn set(ctx: &Context, name: &str) -> Result<()> {
  let mut profiles = Profiles::load(ctx)?;
  if !profiles.profiles.contains_key(name) {
    anyhow::bail!("Profile '{}' not found.", name);
  }
  profiles.active = Some(name.to_string());
  profiles.save(ctx)?;
  println!(
    "{} Profile '{}' activated.",
    style("Set:").green().bold(),
    name
  );
  auto_commit(ctx, &format!("profile set {}", name));
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
    old.unwrap()
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

fn auto_commit(ctx: &Context, msg: &str) {
  if ctx.config.git.auto_commit_enabled() {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}
