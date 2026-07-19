use crate::paths::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn cwd_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn resolve_home_path_tilde_prefix() {
  let home = PathBuf::from("/home/user");
  assert_eq!(resolve_home_path("~/.bashrc", &home), home.join(".bashrc"));
}

#[test]
fn resolve_home_path_home_env_prefix() {
  let home = PathBuf::from("/home/user");
  assert_eq!(
    resolve_home_path("$HOME/.bashrc", &home),
    home.join(".bashrc")
  );
}

#[test]
fn resolve_home_path_absolute_path() {
  let home = PathBuf::from("/home/user");
  assert_eq!(
    resolve_home_path("/etc/config", &home),
    PathBuf::from("/etc/config")
  );
}

#[test]
fn resolve_home_path_relative_path() {
  let home = PathBuf::from("/home/user");
  assert_eq!(
    resolve_home_path("docs/file.txt", &home),
    home.join("docs/file.txt")
  );
}

#[test]
fn resolve_home_path_dot_prefix() {
  let home = PathBuf::from("/home/user");
  assert_eq!(resolve_home_path("./local", &home), home.join("./local"));
}

#[test]
fn resolve_home_path_existing_relative_path_under_cwd() {
  let _guard = cwd_lock().lock().unwrap();
  let old_cwd = std::env::current_dir().unwrap();
  let root = std::env::temp_dir().join(format!(
    "tildr-test-paths-{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let home = root.join("home");
  let documents = home.join("Documents");
  std::fs::create_dir_all(&documents).unwrap();
  std::fs::write(documents.join("document.ods"), "doc").unwrap();

  std::env::set_current_dir(&documents).unwrap();
  assert_eq!(
    resolve_home_path("document.ods", &home),
    documents.join("document.ods")
  );

  std::env::set_current_dir(old_cwd).unwrap();
  std::fs::remove_dir_all(root).ok();
}

#[test]
fn expand_home_absolute_path_unchanged() {
  let result = expand_home("/etc/config");
  assert_eq!(result, PathBuf::from("/etc/config"));
}

#[test]
fn expand_home_relative_path_joins_cwd() {
  let cwd = std::env::current_dir().unwrap();
  let result = expand_home("relative/path");
  assert_eq!(result, cwd.join("relative/path"));
}
