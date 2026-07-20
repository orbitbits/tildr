// This module achieved parallelism through AI, correcting crashes with
// the 'ignore' crate that previously existed using rayon.
// - Suggested by GPT-5.4 (Codex) — OpenAi
//
use crate::ignore::IgnoreMatcher;
use anyhow::Result;
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::{
  ffi::OsStr,
  mem,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  thread,
};
use tildr_utils::fs::should_ignore;

#[derive(Debug, Clone)]
pub struct ManagedEntry {
  /// Profile this file belongs to (e.g. "default", "linux").
  pub profile: String,
  /// Logical home-relative path (e.g. `.bashrc`, `.config/nvim/init.lua`).
  pub relative: PathBuf,
  /// Absolute path on disk.
  pub repo_path: PathBuf,
}

struct ThreadEntries {
  shared: Arc<Mutex<Vec<ManagedEntry>>>,
  local: Vec<ManagedEntry>,
}

impl ThreadEntries {
  fn new(shared: Arc<Mutex<Vec<ManagedEntry>>>) -> Self {
    Self {
      shared,
      local: Vec::new(),
    }
  }

  fn push(&mut self, entry: ManagedEntry) {
    self.local.push(entry);
  }
}

impl Drop for ThreadEntries {
  fn drop(&mut self) {
    if self.local.is_empty() {
      return;
    }

    let mut shared = self
      .shared
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    shared.append(&mut self.local);
  }
}

// TODO: ADICIONAR ARQUIVOS, EXTENSÕES E PASTAS PADRÕES AQUI PARA IGNORAR

pub fn scatildr_repo(repo_path: &Path) -> Result<Vec<ManagedEntry>> {
  if !repo_path.exists() {
    return Ok(Vec::new());
  }

  // Ignored from the user-configured .tildrignore file.
  let ignore = Arc::new(IgnoreMatcher::from_repo(repo_path)?);
  let entries = Arc::new(Mutex::new(Vec::new()));
  let repo_path = repo_path.to_path_buf();
  let threads = thread::available_parallelism().map_or(1, |parallelism| parallelism.get());

  let mut walker = WalkBuilder::new(&repo_path);
  walker
    .hidden(false)
    .parents(false)
    .ignore(false)
    .git_ignore(false)
    .git_global(false)
    .git_exclude(false)
    .threads(threads);

  walker.build_parallel().run(|| {
    let ignore = Arc::clone(&ignore);
    let repo_path = repo_path.clone();
    let entries = Arc::clone(&entries);
    let mut local_entries = ThreadEntries::new(entries);

    Box::new(move |entry| {
      let Ok(entry) = entry else {
        return WalkState::Continue;
      };

      scan_entry(&entry, &repo_path, ignore.as_ref(), &mut local_entries)
    })
  });

  let mut entries = entries
    .lock()
    .map(|mut guard| mem::take(&mut *guard))
    .unwrap_or_else(|poisoned| {
      let mut guard = poisoned.into_inner();
      mem::take(&mut *guard)
    });
  entries.sort_unstable_by(|left, right| {
    left
      .relative
      .cmp(&right.relative)
      .then_with(|| left.profile.cmp(&right.profile))
      .then_with(|| left.repo_path.cmp(&right.repo_path))
  });

  Ok(entries)
}

fn scan_entry(
  entry: &DirEntry,
  repo_path: &Path,
  ignore: &IgnoreMatcher,
  entries: &mut ThreadEntries,
) -> WalkState {
  let path = entry.path();
  let file_type = entry.file_type();
  let is_dir = file_type.is_some_and(|kind| kind.is_dir());

  // Repository metadata is never a managed HOME subtree.
  if let Some(name) = path.file_name()
    && (name == OsStr::new(".git") || name == OsStr::new(".github"))
  {
    return if is_dir {
      WalkState::Skip
    } else {
      WalkState::Continue
    };
  }

  // `.tildr` should prune the subtree, while the remaining generic ignores
  // keep the previous behavior of skipping only the current entry.
  if should_ignore(path) && !is_storage_control_dotfile(path, repo_path) {
    return if is_dir && path.file_name() == Some(OsStr::new(".tildr")) {
      WalkState::Skip
    } else {
      WalkState::Continue
    };
  }

  if ignore.is_ignored(path) {
    return if is_dir {
      WalkState::Skip
    } else {
      WalkState::Continue
    };
  }

  if !file_type.is_some_and(|kind| kind.is_file()) {
    return WalkState::Continue;
  }

  // Process files under common/ (shared files without a profile).
  if let Ok(relative) = path.strip_prefix(repo_path) {
    let relative_str = relative.to_string_lossy();
    if let Some(logical_path) = relative_str.strip_prefix("common/") {
      entries.push(ManagedEntry {
        profile: "common".to_string(),
        relative: PathBuf::from(logical_path),
        repo_path: path.to_path_buf(),
      });
      return WalkState::Continue;
    }
  }

  // Process files under profiles/<name>/ (profile-specific variants).
  // profiles/common/ is supported as a legacy location and maps to common.
  if let Ok(relative) = path.strip_prefix(repo_path) {
    let relative_str = relative.to_string_lossy();
    if let Some(rest) = relative_str.strip_prefix("profiles/")
      && let Some(slash_pos) = rest.find('/')
    {
      let profile_name = &rest[..slash_pos];
      let logical_path = &rest[slash_pos + 1..];
      entries.push(ManagedEntry {
        profile: profile_name.to_string(),
        relative: PathBuf::from(logical_path),
        repo_path: path.to_path_buf(),
      });
      return WalkState::Continue;
    }
  }

  // Process files at the repo root (legacy default profile).
  if let Ok(relative) = path.strip_prefix(repo_path) {
    let first = relative.components().next().map(|part| part.as_os_str());
    if !matches!(
      first,
      Some(name)
        if name == OsStr::new("profiles")
          || name == OsStr::new("common")
          || name == OsStr::new(".git")
          || name == OsStr::new(".github")
          || name == OsStr::new(".tildr")
    ) {
      entries.push(ManagedEntry {
        profile: "default".to_string(),
        relative: relative.to_path_buf(),
        repo_path: path.to_path_buf(),
      });
    }
  }

  WalkState::Continue
}

fn is_storage_control_dotfile(path: &Path, repo_path: &Path) -> bool {
  let Some(name) = path.file_name() else {
    return false;
  };
  if name != OsStr::new(".gitignore") && name != OsStr::new(".tildrignore") {
    return false;
  }

  let Ok(relative) = path.strip_prefix(repo_path) else {
    return false;
  };
  let mut components = relative.components();
  match components.next().map(|component| component.as_os_str()) {
    Some(first) if first == OsStr::new("common") => components.next().is_some(),
    Some(first) if first == OsStr::new("profiles") => {
      components.next().is_some() && components.next().is_some()
    }
    _ => false,
  }
}
