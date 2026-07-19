use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use tildr_core::context::Context;
use tildr_fs::paths::resolve_home_path;
use tildr_repo::{ManagedEntry, scatildr_repo};

use crate::profile::Profiles;

pub enum ResolvedTarget {
  Interactive,
  File(ManagedEntry),
  Dir {
    input: String,
    entries: Vec<ManagedEntry>,
  },
}

/// Entry with profile information for display (list/status).
#[derive(Debug, Clone)]
pub struct ManagedEntryProfile {
  pub profile: String,
  /// Logical home-relative path (e.g. `.bashrc`).
  pub filepath: PathBuf,
  /// Repo-relative path including profile prefix (e.g. `profiles/linux/.bashrc`).
  pub repo_relative: PathBuf,
  pub repo_path: PathBuf,
}

/// Scan all managed entries. The scanner reads files under `profiles/*/`.
pub fn scan_all_entries(_ctx: &Context) -> Result<Vec<ManagedEntry>> {
  let entries = scatildr_repo(&_ctx.repo_path)?;
  Ok(entries)
}

/// Scan all managed entries with profile information for display.
pub fn scan_all_entries_with_profile(ctx: &Context) -> Result<Vec<ManagedEntryProfile>> {
  let entries = scatildr_repo(&ctx.repo_path)?;

  let mut result: Vec<ManagedEntryProfile> = entries
    .into_iter()
    .map(|e| {
      let repo_relative = PathBuf::from("profiles").join(&e.profile).join(&e.relative);
      ManagedEntryProfile {
        profile: e.profile,
        filepath: e.relative,
        repo_relative,
        repo_path: e.repo_path,
      }
    })
    .collect();

  result.sort_by(|a, b| {
    a.profile
      .cmp(&b.profile)
      .then_with(|| a.repo_relative.cmp(&b.repo_relative))
  });
  Ok(result)
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

/// Resolve a logical file path to the actual repo path via the active profile.
/// Falls back to profiles/default/.
pub fn resolve_repo_file(ctx: &Context, relative: &Path) -> Result<PathBuf> {
  let profiles = Profiles::load(ctx)?;
  let file_str = relative.to_string_lossy();
  let resolved = profiles.resolve(&ctx.repo_path, &file_str);

  if resolved.exists() {
    return Ok(resolved);
  }

  bail!("File not found: {}", relative.display())
}
