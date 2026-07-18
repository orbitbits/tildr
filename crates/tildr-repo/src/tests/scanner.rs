use crate::scanner::*;
use std::fs;
use std::path::{Path, PathBuf};

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
