use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use console::style;
use serde::{Deserialize, Serialize};
use tildr_core::context::Context;
use tildr_fs::paths::resolve_home_path;
use tildr_repo::scatildr_repo;
use tildr_utils::{fs::tildr_dir, pager::page_string};
use walkdir::WalkDir;

use crate::apply::{ApplyArgs, run as apply_profile};
use crate::utils::auto_commit::auto_commit;

pub const COMMON_PROFILE: &str = "common";
pub const COMMON_PROFILE_DISPLAY: &str = "no profile";
pub const NO_PROFILE_ALIAS: &str = "no-profile";
pub const DEFAULT_PROFILE: &str = "default";

const MIGRATE_ROOT_EXCLUDES: &[&str] = &[
  ".git",
  ".github",
  ".gitignore",
  ".tildr",
  ".tildrignore",
  COMMON_PROFILE,
  "profiles",
];

pub fn normalize_profile_name(profile: &str) -> &str {
  match profile {
    "no profile" | "no-profile" | "no_profile" => COMMON_PROFILE,
    _ => profile,
  }
}

pub fn display_profile_name(profile: &str) -> &str {
  if profile == COMMON_PROFILE {
    COMMON_PROFILE_DISPLAY
  } else {
    profile
  }
}

pub fn profile_dir(repo_path: &Path, profile: &str) -> PathBuf {
  match profile {
    COMMON_PROFILE => repo_path.join(COMMON_PROFILE),
    DEFAULT_PROFILE => repo_path.to_path_buf(),
    _ => repo_path.join("profiles").join(profile),
  }
}

fn legacy_common_dir(repo_path: &Path) -> PathBuf {
  repo_path.join("profiles").join(COMMON_PROFILE)
}

fn profile_file_label(profile: &str, file: &str) -> String {
  match profile {
    COMMON_PROFILE => format!("{NO_PROFILE_ALIAS}/{file}"),
    DEFAULT_PROFILE => file.to_string(),
    _ => format!("profiles/{profile}/{file}"),
  }
}

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
    let profiles: Profiles = if !path.exists() {
      Self::default()
    } else {
      let data = fs::read_to_string(&path).context("Failed to read profiles file")?;
      serde_json::from_str(&data).context("Failed to parse profiles file")?
    };

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
    if let Some(active) = self
      .active
      .as_deref()
      .filter(|a| *a != DEFAULT_PROFILE && *a != COMMON_PROFILE)
    {
      let candidate = repo_path.join("profiles").join(active).join(file);
      if candidate.exists() {
        return candidate;
      }
    }
    let common_candidate = profile_dir(repo_path, COMMON_PROFILE).join(file);
    if common_candidate.exists() {
      return common_candidate;
    }
    let legacy_common_candidate = legacy_common_dir(repo_path).join(file);
    if legacy_common_candidate.exists() {
      return legacy_common_candidate;
    }
    let default_candidate = repo_path.join("profiles").join(DEFAULT_PROFILE).join(file);
    if default_candidate.exists() {
      return default_candidate;
    }
    // Fallback to repo root for legacy files not yet migrated to profiles/default/
    repo_path.join(file)
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
  if name == DEFAULT_PROFILE || normalize_profile_name(name) == COMMON_PROFILE {
    anyhow::bail!("'{name}' is a reserved name and cannot be used for a profile.");
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
    if from == DEFAULT_PROFILE {
      // Legacy default means all repo entries are tracked.
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

fn logical_file_candidates(ctx: &Context, input: &str) -> Vec<PathBuf> {
  let input_path = Path::new(input);
  let mut candidates = Vec::new();

  let parts: Vec<String> = input_path
    .components()
    .filter_map(|component| match component {
      std::path::Component::Normal(part) => Some(part.to_string_lossy().to_string()),
      _ => None,
    })
    .collect();

  match parts.as_slice() {
    [head, rest @ ..] if normalize_profile_name(head) == COMMON_PROFILE && !rest.is_empty() => {
      candidates.push(rest.iter().collect());
    }
    [profiles, _profile, rest @ ..] if profiles == "profiles" && !rest.is_empty() => {
      candidates.push(rest.iter().collect());
    }
    _ => {}
  }

  let home_path = resolve_home_path(input, &ctx.home_path);
  if let Ok(relative) = home_path.strip_prefix(&ctx.home_path) {
    let relative = relative.to_path_buf();
    if !candidates.iter().any(|candidate| candidate == &relative) {
      candidates.push(relative);
    }
  }

  if !input_path.is_absolute()
    && input != "~"
    && !input.starts_with("~/")
    && input != "$HOME"
    && !input.starts_with("$HOME/")
    && let Ok(cwd) = std::env::current_dir()
    && cwd.starts_with(&ctx.home_path)
  {
    let cwd_path = cwd.join(input_path);
    if let Ok(relative) = cwd_path.strip_prefix(&ctx.home_path) {
      let relative = relative.to_path_buf();
      if !candidates.iter().any(|candidate| candidate == &relative) {
        candidates.push(relative);
      }
    }
  }

  if candidates.is_empty() {
    candidates.push(PathBuf::from(input));
  }

  candidates
}

fn expand_files(ctx: &Context, files: &[String], from_profile: &str) -> Result<Vec<String>> {
  let base = profile_dir(&ctx.repo_path, from_profile);
  let mut result = Vec::new();
  for file in files {
    let candidates = logical_file_candidates(ctx, file);
    let logical = candidates
      .iter()
      .find(|candidate| base.join(candidate).exists() || ctx.repo_path.join(candidate).exists())
      .unwrap_or(&candidates[0]);
    let profile_path = base.join(logical);
    let path = if profile_path.exists() {
      profile_path
    } else {
      ctx.repo_path.join(logical)
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
      result.push(logical.to_string_lossy().to_string());
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
  let from = normalize_profile_name(from);
  let to = normalize_profile_name(to);

  if from == to {
    println!("{}", style("Source and destination are the same.").dim());
    return Ok(());
  }

  let profiles = Profiles::load(ctx)?;

  if from != DEFAULT_PROFILE && from != COMMON_PROFILE && !profiles.profiles.contains_key(from) {
    anyhow::bail!("Profile '{}' not found.", from);
  }

  if to != DEFAULT_PROFILE && to != COMMON_PROFILE && !profiles.profiles.contains_key(to) {
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

  let from_profile = from != DEFAULT_PROFILE;
  let to_profile = to != DEFAULT_PROFILE;
  let mut count = 0;

  for file in &file_list {
    let src = profile_dir(&ctx.repo_path, from).join(file);

    if !src.exists() {
      println!("  {} {} (file not found)", style("Skipped:").yellow(), file);
      continue;
    }

    let dst = if to_profile {
      let dir = profile_dir(&ctx.repo_path, to);
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
    let direction = format!(
      "{} -> {}",
      profile_file_label(from, file),
      profile_file_label(to, file)
    );
    println!("  {} {}", action_fn(action), direction);
  }

  if remove_source && from_profile && from != COMMON_PROFILE {
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

  let profile_path = ctx.repo_path.join("profiles").join(name);
  let common_dir = profile_dir(&ctx.repo_path, COMMON_PROFILE);

  // Restore files to common/ if they don't exist there.
  if profile_path.exists() {
    for entry in WalkDir::new(&profile_path)
      .into_iter()
      .filter_map(|e| e.ok())
    {
      if entry.file_type().is_file() {
        let relative = entry
          .path()
          .strip_prefix(&profile_path)
          .unwrap_or(entry.path());
        let target = common_dir.join(relative);
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

  if profile_path.exists() {
    fs::remove_dir_all(&profile_path)?;
  }

  profiles.save(ctx)?;
  println!(
    "{} Profile '{name}' deleted.",
    style("Deleted:").red().bold()
  );
  relink_effective_profile(ctx)?;
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
    let name = normalize_profile_name(name);
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
  relink_effective_profile(ctx)?;
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
  relink_effective_profile(ctx)?;
  auto_commit(ctx, "profile unset");
  Ok(())
}

fn relink_effective_profile(ctx: &Context) -> Result<()> {
  apply_profile(
    ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: false,
    },
  )
}

fn current(ctx: &Context) -> Result<()> {
  let profiles = Profiles::load(ctx)?;
  match &profiles.active {
    Some(name) => println!("Active profile: {}", style(name).cyan()),
    None => println!("No profile is currently active. Using no-profile files."),
  }
  Ok(())
}

fn migrate(ctx: &Context, dry_run: bool) -> Result<()> {
  // Find files at repo root that should be in common/
  let root_entries: Vec<_> = fs::read_dir(&ctx.repo_path)?
    .filter_map(|e| e.ok())
    .filter(|e| {
      let name = e.file_name().to_string_lossy().to_string();
      !MIGRATE_ROOT_EXCLUDES.contains(&name.as_str())
    })
    .collect();

  let common_dir = profile_dir(&ctx.repo_path, COMMON_PROFILE);

  let mut count = 0;

  for entry in &root_entries {
    let name = entry.file_name().to_string_lossy().to_string();

    // Skip if already exists in common/
    let target = common_dir.join(&name);
    if target.exists() {
      println!(
        "  {} {} (already in common/)",
        style("Skipped:").dim(),
        name
      );
      continue;
    }

    if dry_run {
      println!(
        "  {} {} -> common/{name}",
        style("Would migrate:").cyan(),
        name
      );
    } else {
      let source = entry.path();
      if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
      }
      fs::rename(&source, &target)?;
      println!("  {} {} -> common/{name}", style("Migrated:").green(), name);
    }
    count += 1;
  }

  let legacy_common_dir = legacy_common_dir(&ctx.repo_path);
  if legacy_common_dir.exists() {
    for entry in WalkDir::new(&legacy_common_dir)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().is_file())
    {
      let relative = entry
        .path()
        .strip_prefix(&legacy_common_dir)
        .unwrap_or(entry.path());
      let target = common_dir.join(relative);

      if target.exists() {
        println!(
          "  {} profiles/common/{} (already in common/)",
          style("Skipped:").dim(),
          relative.display()
        );
        continue;
      }

      if dry_run {
        println!(
          "  {} profiles/common/{} -> common/{}",
          style("Would migrate:").cyan(),
          relative.display(),
          relative.display()
        );
      } else {
        if let Some(parent) = target.parent() {
          fs::create_dir_all(parent)?;
        }
        fs::rename(entry.path(), &target)?;
        println!(
          "  {} profiles/common/{} -> common/{}",
          style("Migrated:").green(),
          relative.display(),
          relative.display()
        );
      }

      count += 1;
    }

    let has_remaining_files = WalkDir::new(&legacy_common_dir)
      .into_iter()
      .filter_map(|e| e.ok())
      .any(|e| e.file_type().is_file());
    if !dry_run && !has_remaining_files {
      fs::remove_dir_all(&legacy_common_dir)?;
    }
  }

  if count == 0 {
    println!("{}", style("Nothing to migrate.").dim());
    return Ok(());
  }

  if !dry_run {
    auto_commit(ctx, &format!("migrate {count} file(s) to common/"));
  }

  if dry_run {
    println!(
      "\n{} {count} file(s) would be moved to common/ (dry run)",
      style("Would migrate:").cyan()
    );
  } else {
    println!(
      "\n{} {count} file(s) migrated to common/",
      style("Migrated:").green().bold()
    );
  }

  Ok(())
}
