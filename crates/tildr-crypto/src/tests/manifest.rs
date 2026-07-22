use crate::manifest::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir() -> PathBuf {
  static NEXT_ID: AtomicU64 = AtomicU64::new(0);

  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  for _ in 0..100 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
      "tildr-test-manifest-{}-{nanos}-{id}",
      std::process::id()
    ));

    if fs::create_dir(&dir).is_ok() {
      return dir;
    }
  }

  panic!("failed to create unique test directory");
}

fn setup_manifest(dir: &Path) -> EncryptManifest {
  let tildr = dir.join(".tildr");
  fs::create_dir_all(&tildr).unwrap();
  let path = tildr.join("encrypted-items");
  fs::write(&path, "").unwrap();
  EncryptManifest { path }
}

#[test]
fn entries_returns_empty_when_file_missing() {
  let manifest = EncryptManifest {
    path: PathBuf::from("/tmp/nonexistent-tildr-test-file"),
  };
  assert!(manifest.entries().unwrap().is_empty());
}

#[test]
fn add_entry_appends_to_file() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  manifest.add(".bashrc").unwrap();
  let entries = manifest.entries().unwrap();
  assert_eq!(entries, vec![".bashrc"]);
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn add_entry_is_idempotent() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  manifest.add(".bashrc").unwrap();
  manifest.add(".bashrc").unwrap();
  let entries = manifest.entries().unwrap();
  assert_eq!(entries, vec![".bashrc"]);
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn add_multiple_entries() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  manifest.add(".bashrc").unwrap();
  manifest.add(".zshrc").unwrap();
  let entries = manifest.entries().unwrap();
  assert_eq!(entries, vec![".bashrc", ".zshrc"]);
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn remove_entry_returns_true_and_removes() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  manifest.add(".bashrc").unwrap();
  assert!(manifest.remove(".bashrc").unwrap());
  assert!(manifest.entries().unwrap().is_empty());
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn remove_nonexistent_entry_returns_false() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  assert!(!manifest.remove(".bashrc").unwrap());
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn exists_returns_true_when_file_exists() {
  let dir = temp_dir();
  let manifest = setup_manifest(&dir);
  assert!(manifest.exists());
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn exists_returns_false_when_file_missing() {
  let manifest = EncryptManifest {
    path: PathBuf::from("/tmp/nonexistent-tildr-test"),
  };
  assert!(!manifest.exists());
}
