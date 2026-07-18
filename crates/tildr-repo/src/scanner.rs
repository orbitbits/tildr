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
  pub relative: PathBuf,
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
  entries.sort_unstable_by(|left, right| left.relative.cmp(&right.relative));

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

  if path.file_name() == Some(OsStr::new(".git")) {
    return if is_dir {
      WalkState::Skip
    } else {
      WalkState::Continue
    };
  }

  // `.tildr` should prune the subtree, while the remaining generic ignores
  // keep the previous behavior of skipping only the current entry.
  if should_ignore(path) {
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

  if let Ok(relative) = path.strip_prefix(repo_path) {
    entries.push(ManagedEntry {
      relative: relative.to_path_buf(),
      repo_path: path.to_path_buf(),
    });
  }

  WalkState::Continue
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn temp_repo() -> PathBuf {
    let nanos = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let dir = std::env::temp_dir().join(format!("tildr-test-scanner-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn scan_empty_directory() {
    let dir = temp_repo();
    let entries = scatildr_repo(&dir).unwrap();
    assert!(entries.is_empty());
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn scan_nonexistent_directory() {
    let entries = scatildr_repo(Path::new("/tmp/nonexistent-tildr-scan")).unwrap();
    assert!(entries.is_empty());
  }

  #[test]
  fn scan_ignores_git_directory() {
    let dir = temp_repo();
    fs::create_dir(dir.join(".git")).unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();
    let entries = scatildr_repo(&dir).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn scan_ignores_git_files() {
    let dir = temp_repo();
    fs::write(dir.join(".git"), "gitfile").unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();
    let entries = scatildr_repo(&dir).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn scan_skips_tildr_directory() {
    let dir = temp_repo();
    fs::create_dir_all(dir.join(".tildr")).unwrap();
    fs::write(dir.join(".tildr").join("meta.toml"), "key=val").unwrap();
    fs::write(dir.join("file.txt"), "content").unwrap();
    let entries = scatildr_repo(&dir).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn scan_returns_sorted_entries() {
    let dir = temp_repo();
    fs::write(dir.join("z.txt"), "z").unwrap();
    fs::write(dir.join("a.txt"), "a").unwrap();
    fs::write(dir.join("m.txt"), "m").unwrap();
    let entries = scatildr_repo(&dir).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].relative, PathBuf::from("a.txt"));
    assert_eq!(entries[1].relative, PathBuf::from("m.txt"));
    assert_eq!(entries[2].relative, PathBuf::from("z.txt"));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn scan_handles_nested_directories() {
    let dir = temp_repo();
    fs::create_dir_all(dir.join("config")).unwrap();
    fs::write(dir.join("config").join("settings.json"), "{}").unwrap();
    fs::write(dir.join(".bashrc"), "export").unwrap();
    let entries = scatildr_repo(&dir).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].relative, PathBuf::from(".bashrc"));
    assert_eq!(entries[1].relative, PathBuf::from("config/settings.json"));
    fs::remove_dir_all(&dir).ok();
  }
}
