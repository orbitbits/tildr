use anyhow::{Result, bail};
use console::style;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use tildr_core::context::Context;
use tildr_fs::paths::{normalize_lexically, resolve_home_path};
use tildr_repo::{ManagedEntry, scatildr_repo};

use crate::profile::{
  COMMON_PROFILE, DEFAULT_PROFILE, NO_PROFILE_ALIAS, Profiles, display_profile_name,
  normalize_profile_name, profile_dir, validate_profile_component,
};

pub enum ResolvedTarget {
  Interactive,
  File(ManagedEntry),
  Dir {
    input: String,
    entries: Vec<ManagedEntry>,
  },
}

/// Result of resolving a logical file path against the profile hierarchy.
pub enum FileResolution {
  /// File found — entry carries the profile name and physical path.
  Found(ManagedEntry),
  /// File exists in multiple profiles but not in the active/default one.
  /// Contains the list of profile names where it was found.
  AmbiguousAcrossProfiles(Vec<String>),
  /// File does not exist in any profile.
  NotManaged,
}

/// Entry with profile information for display (list/status).
#[derive(Debug, Clone)]
pub struct ManagedEntryProfile {
  pub profile: String,
  /// Logical home-relative path (e.g. `.bashrc`).
  pub filepath: PathBuf,
  /// Repo-relative path on disk (e.g. `common/.bashrc`, `profiles/linux/.bashrc`).
  pub repo_relative: PathBuf,
  pub repo_path: PathBuf,
}

/// Scan all managed entries. The scanner reads files under `common/` and `profiles/*/`.
pub fn scan_all_entries(ctx: &Context) -> Result<Vec<ManagedEntry>> {
  let entries = scatildr_repo(&ctx.repo_path)?;
  Ok(entries)
}

fn with_profile(ctx: &Context, entry: ManagedEntry) -> ManagedEntryProfile {
  let repo_relative = entry
    .repo_path
    .strip_prefix(&ctx.repo_path)
    .unwrap_or(&entry.repo_path)
    .to_path_buf();

  ManagedEntryProfile {
    profile: entry.profile,
    filepath: entry.relative,
    repo_relative,
    repo_path: entry.repo_path,
  }
}

/// Scan all managed entries with profile information for display.
pub fn scan_all_entries_with_profile(ctx: &Context) -> Result<Vec<ManagedEntryProfile>> {
  let entries = scatildr_repo(&ctx.repo_path)?;

  let mut result: Vec<ManagedEntryProfile> = entries
    .into_iter()
    .map(|entry| with_profile(ctx, entry))
    .collect();

  result.sort_by(|a, b| {
    a.profile
      .cmp(&b.profile)
      .then_with(|| a.repo_relative.cmp(&b.repo_relative))
  });
  Ok(result)
}

pub fn select_effective_entry(
  repo_path: &Path,
  profiles: &Profiles,
  entries: &[ManagedEntryProfile],
) -> Option<ManagedEntryProfile> {
  let first = entries.first()?;
  let file_str = first.filepath.to_string_lossy();
  let expected = profiles.resolve(repo_path, &file_str);

  entries
    .iter()
    .find(|entry| entry.repo_path == expected)
    .cloned()
}

pub fn effective_entries(
  repo_path: &Path,
  profiles: &Profiles,
  by_filepath: &std::collections::HashMap<PathBuf, Vec<ManagedEntryProfile>>,
) -> Vec<ManagedEntryProfile> {
  let mut entries: Vec<ManagedEntryProfile> = by_filepath
    .values()
    .filter_map(|entries| select_effective_entry(repo_path, profiles, entries))
    .collect();

  entries.sort_by(|left, right| left.filepath.cmp(&right.filepath));
  entries
}

pub fn scan_effective_entries(ctx: &Context) -> Result<Vec<ManagedEntry>> {
  let profiles = Profiles::load(ctx)?;
  let mut by_filepath: HashMap<PathBuf, Vec<ManagedEntryProfile>> = HashMap::new();

  for entry in scan_all_entries(ctx)? {
    let entry = with_profile(ctx, entry);
    by_filepath
      .entry(entry.filepath.clone())
      .or_default()
      .push(entry);
  }

  Ok(
    effective_entries(&ctx.repo_path, &profiles, &by_filepath)
      .into_iter()
      .map(|entry| ManagedEntry {
        profile: entry.profile,
        relative: entry.filepath,
        repo_path: entry.repo_path,
      })
      .collect(),
  )
}

pub fn to_relative(ctx: &Context, path: &Path) -> Result<PathBuf> {
  Ok(path.strip_prefix(&ctx.home_path)?.to_path_buf())
}

/// Resolve a logical file path to a physical `ManagedEntry`, following the
/// profile hierarchy: active → default → ambiguous error.
///
/// When `profile_override` is `Some`, skip the hierarchy and resolve
/// directly in the specified profile.
pub fn resolve_logical_file(
  ctx: &Context,
  relative: &Path,
  profile_override: Option<&str>,
) -> Result<FileResolution> {
  let profiles = Profiles::load(ctx)?;
  let file_str = relative.to_string_lossy();

  // --- Explicit --profile flag: go straight to that profile ---
  if let Some(name) = profile_override {
    let name = normalize_profile_name(name);
    let mut candidate = profile_dir(&ctx.repo_path, name).join(relative);
    if !candidate.exists() && name == COMMON_PROFILE {
      let legacy_candidate = ctx
        .repo_path
        .join("profiles")
        .join(COMMON_PROFILE)
        .join(relative);
      if legacy_candidate.exists() {
        candidate = legacy_candidate;
      }
    }

    if candidate.is_file() {
      return Ok(FileResolution::Found(ManagedEntry {
        profile: name.to_string(),
        relative: relative.to_path_buf(),
        repo_path: candidate,
      }));
    }
    if candidate.is_dir() {
      return Ok(FileResolution::NotManaged);
    }
    bail!(
      "File '{}' not found in profile '{}'.",
      relative.display(),
      name
    );
  }

  // --- Standard hierarchy: active → default ---
  let resolved_path = profiles.resolve(&ctx.repo_path, &file_str);
  if resolved_path.is_file() {
    // Deduce profile name from the resolved path
    let profile_name = if resolved_path.starts_with(profile_dir(&ctx.repo_path, COMMON_PROFILE)) {
      COMMON_PROFILE.to_string()
    } else {
      resolved_path
        .strip_prefix(ctx.repo_path.join("profiles"))
        .ok()
        .and_then(|p| p.components().next())
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| DEFAULT_PROFILE.to_string())
    };

    return Ok(FileResolution::Found(ManagedEntry {
      profile: profile_name,
      relative: relative.to_path_buf(),
      repo_path: resolved_path,
    }));
  }

  // --- Ambiguity check: exists in other profile(s)? ---
  let entries = scan_all_entries(ctx)?;
  let others: Vec<String> = entries
    .iter()
    .filter(|e| e.relative == relative)
    .map(|e| display_profile_name(&e.profile).to_string())
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();

  if !others.is_empty() {
    return Ok(FileResolution::AmbiguousAcrossProfiles(others));
  }

  Ok(FileResolution::NotManaged)
}

pub fn storage_to_home_relative(repo_path: &Path, path: &Path) -> PathBuf {
  let relative = path.strip_prefix(repo_path).unwrap_or(path);
  let normalized = normalize_lexically(relative);
  let parts: Vec<_> = normalized
    .components()
    .filter_map(|component| match component {
      Component::Normal(part) => Some(part.to_os_string()),
      _ => None,
    })
    .collect();

  match parts.as_slice() {
    [head, rest @ ..]
      if (head == COMMON_PROFILE || head == NO_PROFILE_ALIAS || head == "no_profile")
        && !rest.is_empty() =>
    {
      rest.iter().collect()
    }
    [profiles, _profile, rest @ ..] if profiles == "profiles" && !rest.is_empty() => {
      rest.iter().collect()
    }
    _ => normalized,
  }
}

fn normalize_storage_target(ctx: &Context, target: &str) -> (String, Option<String>) {
  let path = Path::new(target);
  let relative = if path.is_absolute() {
    let Ok(relative) = path.strip_prefix(&ctx.repo_path) else {
      return (target.to_string(), None);
    };
    relative
  } else {
    path
  };
  let normalized = normalize_lexically(relative);
  let parts: Vec<String> = normalized
    .components()
    .filter_map(|component| match component {
      Component::Normal(part) => Some(part.to_string_lossy().to_string()),
      _ => None,
    })
    .collect();

  let inferred_profile = match parts.as_slice() {
    [head, rest @ ..] if normalize_profile_name(head) == COMMON_PROFILE && !rest.is_empty() => {
      Some(COMMON_PROFILE.to_string())
    }
    [profiles, profile, rest @ ..] if profiles == "profiles" && !rest.is_empty() => {
      Some(normalize_profile_name(profile).to_string())
    }
    _ => None,
  };

  if inferred_profile.is_some() {
    (
      storage_to_home_relative(&ctx.repo_path, &normalized)
        .display()
        .to_string(),
      inferred_profile,
    )
  } else {
    (target.to_string(), None)
  }
}

fn candidate_home_paths(ctx: &Context, target: &str) -> Vec<PathBuf> {
  let input_path = Path::new(target);
  let mut candidates = vec![resolve_home_path(target, &ctx.home_path)];

  if !input_path.is_absolute()
    && target != "~"
    && !target.starts_with("~/")
    && target != "$HOME"
    && !target.starts_with("$HOME/")
  {
    if let Ok(cwd) = std::env::current_dir()
      && cwd.starts_with(&ctx.home_path)
    {
      candidates.push(normalize_lexically(&cwd.join(input_path)));
    }

    candidates.push(ctx.home_path.join(input_path));
  }

  candidates.dedup();
  candidates
}

pub fn logical_home_candidates(ctx: &Context, target: &str) -> Result<Vec<PathBuf>> {
  let mut relatives = Vec::new();
  for candidate in candidate_home_paths(ctx, target) {
    let relative = to_relative(ctx, &candidate)?;
    if !relatives.contains(&relative) {
      relatives.push(relative);
    }
  }
  Ok(relatives)
}

pub fn resolve_target(
  ctx: &Context,
  target: Option<String>,
  profile_override: Option<&str>,
) -> Result<ResolvedTarget> {
  let Some(target) = target else {
    return Ok(ResolvedTarget::Interactive);
  };

  if let Some(profile) = profile_override {
    let profile = normalize_profile_name(profile);
    validate_profile_component(profile)?;
    if profile != COMMON_PROFILE
      && profile != DEFAULT_PROFILE
      && !Profiles::load(ctx)?.profiles.contains_key(profile)
    {
      bail!("Profile '{profile}' not found.");
    }
  }

  let (target, inferred_profile) = normalize_storage_target(ctx, &target);
  let effective_profile = profile_override
    .map(normalize_profile_name)
    .map(str::to_string)
    .or(inferred_profile);

  let relatives = logical_home_candidates(ctx, &target)?;
  let mut ambiguous: Option<(PathBuf, Vec<String>)> = None;

  for relative in &relatives {
    // Single-file resolution uses the profile hierarchy
    match resolve_logical_file(ctx, relative, effective_profile.as_deref())? {
      FileResolution::Found(entry) => return Ok(ResolvedTarget::File(entry)),
      FileResolution::AmbiguousAcrossProfiles(profiles) => {
        ambiguous.get_or_insert((relative.clone(), profiles));
      }
      FileResolution::NotManaged => {}
    }
  }

  // Fallback: directory resolution (scan all entries for prefix match)
  let entries = scan_all_entries(ctx)?;

  let mut dir_entries: Vec<ManagedEntry> = entries
    .into_iter()
    .filter(|entry| {
      relatives
        .iter()
        .any(|relative| entry.relative.starts_with(relative))
    })
    .filter(|entry| {
      effective_profile
        .as_deref()
        .is_none_or(|profile| entry.profile == profile)
    })
    .collect();

  if effective_profile.is_none() {
    let profiles = Profiles::load(ctx)?;
    let mut by_filepath: HashMap<PathBuf, Vec<ManagedEntryProfile>> = HashMap::new();
    for entry in dir_entries {
      let entry = with_profile(ctx, entry);
      by_filepath
        .entry(entry.filepath.clone())
        .or_default()
        .push(entry);
    }
    dir_entries = effective_entries(&ctx.repo_path, &profiles, &by_filepath)
      .into_iter()
      .map(|entry| ManagedEntry {
        profile: entry.profile,
        relative: entry.filepath,
        repo_path: entry.repo_path,
      })
      .collect();
  }

  if dir_entries.is_empty() {
    if let Some((relative, profiles)) = ambiguous {
      let profiles_str = profiles.join(", ");
      let hint = if let Some(active) = Profiles::load(ctx)?.active.as_deref() {
        format!(
          "Active profile is '{}'. Use --profile <name>, or run `tildr profile set <name>` before.",
          active
        )
      } else {
        "No profile is active. Use --profile <name>, or run `tildr profile set <name>` first."
          .to_string()
      };
      bail!(
        "File '{}' is ambiguous — exists in profiles: {}. {}",
        relative.display(),
        style(profiles_str).cyan(),
        hint
      );
    }

    let display = relatives
      .first()
      .map(|relative| relative.display().to_string())
      .unwrap_or_else(|| target.clone());
    bail!("Target is not managed: {display}");
  }

  Ok(ResolvedTarget::Dir {
    input: target.trim_end_matches('/').to_string(),
    entries: dir_entries,
  })
}

pub fn resolve_targets(
  ctx: &Context,
  targets: &[String],
  profile_override: Option<&str>,
) -> Result<Vec<ResolvedTarget>> {
  let mut resolved = Vec::with_capacity(targets.len());

  for target in targets {
    resolved.push(resolve_target(ctx, Some(target.clone()), profile_override)?);
  }

  Ok(resolved)
}

pub fn collect_resolved_entries<F>(
  resolved_targets: &[ResolvedTarget],
  mut confirm_dir: F,
) -> Result<Vec<ManagedEntry>>
where
  F: FnMut(&str) -> Result<()>,
{
  let mut entries = Vec::new();
  let mut seen = HashSet::new();

  for resolved in resolved_targets {
    match resolved {
      ResolvedTarget::Interactive => {}
      ResolvedTarget::File(entry) => {
        if seen.insert(entry.relative.clone()) {
          entries.push(entry.clone());
        }
      }
      ResolvedTarget::Dir {
        input,
        entries: dir_entries,
      } => {
        confirm_dir(input)?;

        for entry in dir_entries {
          if seen.insert(entry.relative.clone()) {
            entries.push(entry.clone());
          }
        }
      }
    }
  }

  Ok(entries)
}
