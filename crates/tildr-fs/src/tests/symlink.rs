use crate::symlink::*;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir() -> PathBuf {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  std::env::temp_dir().join(format!("tildr-test-symlink-{nanos}"))
}

#[test]
fn create_and_check_symlink() {
  let dir = temp_dir();
  fs::create_dir_all(&dir).unwrap();
  let target = dir.join("target.txt");
  let link = dir.join("link.txt");
  fs::write(&target, "hello").unwrap();

  create_symlink(&target, &link).unwrap();
  assert!(is_symlink(&link));
  assert_eq!(symlink_target(&link), Some(target.clone()));
  assert!(is_symlink_to(&link, &target));

  fs::remove_dir_all(&dir).ok();
}

#[test]
fn is_symlink_returns_false_for_regular_file() {
  let dir = temp_dir();
  fs::create_dir_all(&dir).unwrap();
  let file = dir.join("regular.txt");
  fs::write(&file, "content").unwrap();
  assert!(!is_symlink(&file));
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn is_symlink_returns_false_for_nonexistent_path() {
  assert!(!is_symlink(Path::new("/nonexistent/path")));
}

#[test]
fn symlink_target_returns_none_for_regular_file() {
  let dir = temp_dir();
  fs::create_dir_all(&dir).unwrap();
  let file = dir.join("target.txt");
  fs::write(&file, "content").unwrap();
  assert!(symlink_target(&file).is_none());
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn is_symlink_to_false_when_target_differs() {
  let dir = temp_dir();
  fs::create_dir_all(&dir).unwrap();
  let target = dir.join("real_target.txt");
  let wrong = dir.join("wrong_target.txt");
  let link = dir.join("link.txt");
  fs::write(&target, "real").unwrap();
  fs::write(&wrong, "wrong").unwrap();

  create_symlink(&target, &link).unwrap();
  assert!(!is_symlink_to(&link, &wrong));

  fs::remove_dir_all(&dir).ok();
}
