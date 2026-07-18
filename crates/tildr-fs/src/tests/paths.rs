use crate::paths::*;
use std::path::PathBuf;

#[test]
fn resolve_home_path_tilde_prefix() {
  let home = PathBuf::from("/home/user");
  assert_eq!(resolve_home_path("~/.bashrc", &home), home.join(".bashrc"));
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
