use anyhow::{Result, bail};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tildr_core::context::Context;
use tildr_fs::paths::resolve_home_path;
use tildr_repo::{ManagedEntry, scatildr_repo};
use walkdir::WalkDir;

use crate::profile::Profiles;

pub enum ResolvedTarget {
  Interactive,
  File(ManagedEntry),
  Dir {
    input: String,
    entries: Vec<ManagedEntry>,
  },
}

/// Entry with profile information.
#[derive(Debug, Clone)]
pub struct ManagedEntryProfile {
  pub profile: String,
  pub filepath: PathBuf,
  pub repo_path: PathBuf,
}

pub fn scan_all_entries(ctx: &Context) -> Result<Vec<ManagedEntry>> {
  let mut entries = scatildr_repo(&ctx.repo_path)?;

  // Include profile variant files that don't exist at the repo root.
  let profiles = Profiles::load(ctx)?;
  let root_rel: HashSet<PathBuf> = entries.iter().map(|e| e.relative.clone()).collect();

  let profiles_dir = ctx.repo_path.join("profiles");
  if profiles_dir.is_dir() {
    for profile_name in profiles.profiles.keys() {
      let dir = profiles_dir.join(profile_name);
      if !dir.is_dir() {
        continue;
      }
      for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
      {
        let full = entry.path();
        let relative_in_profile = full.strip_prefix(&dir).unwrap_or(full).to_path_buf();

        // Only include if this logical path is NOT already at the repo root.
        if !root_rel.contains(&relative_in_profile) {
          entries.push(ManagedEntry {
            relative: relative_in_profile,
            repo_path: full.to_path_buf(),
          });
        }
      }
    }
  }

  entries.sort_by(|left, right| left.relative.cmp(&right.relative));
  Ok(entries)
}

/// Scan all managed entries with profile information.
/// Returns entries where `filepath` is the logical path (without profile prefix)
/// and `profile` indicates which profile it belongs to ("default" for root files).
pub fn scan_all_entries_with_profile(ctx: &Context) -> Result<Vec<ManagedEntryProfile>> {
  let mut entries = Vec::new();

  // Root files (default profile)
  let root_entries = scatildr_repo(&ctx.repo_path)?;
  let root_rel: HashSet<PathBuf> = root_entries.iter().map(|e| e.relative.clone()).collect();

  for entry in root_entries {
    entries.push(ManagedEntryProfile {
      profile: "default".to_string(),
      filepath: entry.relative.clone(),
      repo_path: entry.repo_path,
    });
  }

  // Profile variant files
  let profiles = Profiles::load(ctx)?;
  let profiles_dir = ctx.repo_path.join("profiles");
  if profiles_dir.is_dir() {
    for profile_name in profiles.profiles.keys() {
      let dir = profiles_dir.join(profile_name);
      if !dir.is_dir() {
        continue;
      }
      for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
      {
        let full = entry.path();
        let relative_in_profile = full.strip_prefix(&dir).unwrap_or(full).to_path_buf();

        // Only include if this logical path is NOT already at the repo root.
        if !root_rel.contains(&relative_in_profile) {
          entries.push(ManagedEntryProfile {
            profile: profile_name.clone(),
            filepath: relative_in_profile,
            repo_path: full.to_path_buf(),
          });
        }
      }
    }
  }

  entries.sort_by(|a, b| {
    a.profile
      .cmp(&b.profile)
      .then_with(|| a.filepath.cmp(&b.filepath))
  });
  Ok(entries)
}

pub fn to_relative(ctx: &Context, path: &Path) -> Result<PathBuf> {
  Ok(path.strip_prefix(&ctx.home_path)?.to_path_buf())
}

pub fn resolve_target(ctx: &Context, target: Option<String>) -> Result<ResolvedTarget> {
  let Some(target) = target else {
    return Ok(ResolvedTarget::Interactive);
  };

  let home_path = resolve_home_path(&target, &ctx.home_path);
  let relative = to_relative(ctx, &home_path)?;
  let mut entries = scan_all_entries(ctx)?;

  if let Some(index) = entries.iter().position(|entry| entry.relative == relative) {
    return Ok(ResolvedTarget::File(entries.swap_remove(index)));
  }

  let dir_entries: Vec<ManagedEntry> = entries
    .into_iter()
    .filter(|entry| entry.relative.starts_with(&relative))
    .collect();

  if dir_entries.is_empty() {
    bail!("Target is not managed: {}", relative.display());
  }

  Ok(ResolvedTarget::Dir {
    input: target.trim_end_matches('/').to_string(),
    entries: dir_entries,
  })
}

pub fn resolve_targets(ctx: &Context, targets: &[String]) -> Result<Vec<ResolvedTarget>> {
  let mut resolved = Vec::with_capacity(targets.len());

  for target in targets {
    resolved.push(resolve_target(ctx, Some(target.clone()))?);
  }

  Ok(resolved)
}

/// Resolve a logical file path to the actual repo path, checking profiles.
pub fn resolve_repo_file(ctx: &Context, relative: &Path) -> Result<PathBuf> {
  let repo_file = ctx.repo_path.join(relative);
  if repo_file.exists() {
    return Ok(repo_file);
  }

  let profiles_dir = ctx.repo_path.join("profiles");
  if profiles_dir.is_dir() {
    for entry in std::fs::read_dir(&profiles_dir)
      .into_iter()
      .flatten()
      .filter_map(|e| e.ok())
    {
      let variant = entry.path().join(relative);
      if variant.is_file() {
        return Ok(variant);
      }
    }
  }

  bail!("File not found: {}", relative.display())
}
