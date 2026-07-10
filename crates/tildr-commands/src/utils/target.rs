use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use tildr_core::context::Context;
use tildr_fs::paths::resolve_home_path;
use tildr_repo::{ManagedEntry, scatildr_repo};

pub enum ResolvedTarget {
  Interactive,
  File(ManagedEntry),
  Dir {
    input: String,
    entries: Vec<ManagedEntry>,
  },
}

pub fn scan_all_entries(ctx: &Context) -> Result<Vec<ManagedEntry>> {
  let mut entries = scatildr_repo(&ctx.repo_path)?;
  entries.sort_by(|left, right| left.relative.cmp(&right.relative));
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
