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

fn profiles_default(dir: &Path) -> PathBuf {
  let p = dir.join("profiles/default");
  fs::create_dir_all(&p).unwrap();
  p
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
  let pd = profiles_default(&dir);
  fs::create_dir(dir.join(".git")).unwrap();
  fs::write(pd.join("file.txt"), "content").unwrap();
  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
  assert_eq!(entries[0].profile, "default");
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_ignores_git_files() {
  let dir = temp_repo();
  let pd = profiles_default(&dir);
  fs::write(dir.join(".git"), "gitfile").unwrap();
  fs::write(pd.join("file.txt"), "content").unwrap();
  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_keeps_legacy_git_dotfiles() {
  let dir = temp_repo();
  fs::write(dir.join(".gitconfig"), "[user]").unwrap();
  fs::create_dir_all(dir.join(".github/workflows")).unwrap();
  fs::write(dir.join(".github/workflows/ci.yml"), "name: ci").unwrap();

  let entries = scatildr_repo(&dir).unwrap();

  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].relative, PathBuf::from(".gitconfig"));
  assert_eq!(entries[0].profile, "default");

  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_distinguishes_root_control_files_from_managed_dotfiles() {
  let dir = temp_repo();
  fs::create_dir_all(dir.join("common")).unwrap();
  fs::create_dir_all(dir.join("profiles/linux")).unwrap();
  fs::write(dir.join(".gitignore"), "target/").unwrap();
  fs::write(dir.join(".tildrignore"), "cache/").unwrap();
  fs::write(dir.join("._.gitignore"), "mac metadata").unwrap();
  fs::write(dir.join("common/.gitignore"), "*.log").unwrap();
  fs::write(dir.join("profiles/linux/.tildrignore"), "linux-cache/").unwrap();

  let entries = scatildr_repo(&dir).unwrap();

  assert_eq!(entries.len(), 2);
  assert!(
    entries
      .iter()
      .any(|entry| { entry.profile == "common" && entry.relative == Path::new(".gitignore") })
  );
  assert!(
    entries
      .iter()
      .any(|entry| { entry.profile == "linux" && entry.relative == Path::new(".tildrignore") })
  );

  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_skips_tildr_directory() {
  let dir = temp_repo();
  let pd = profiles_default(&dir);
  fs::create_dir_all(dir.join(".tildr")).unwrap();
  fs::write(dir.join(".tildr").join("meta.toml"), "key=val").unwrap();
  fs::write(pd.join("file.txt"), "content").unwrap();
  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].relative, PathBuf::from("file.txt"));
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_returns_sorted_entries() {
  let dir = temp_repo();
  let pd = profiles_default(&dir);
  fs::write(pd.join("z.txt"), "z").unwrap();
  fs::write(pd.join("a.txt"), "a").unwrap();
  fs::write(pd.join("m.txt"), "m").unwrap();
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
  let pd = profiles_default(&dir);
  fs::create_dir_all(pd.join("config")).unwrap();
  fs::write(pd.join("config").join("settings.json"), "{}").unwrap();
  fs::write(pd.join(".bashrc"), "export").unwrap();
  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 2);
  assert_eq!(entries[0].relative, PathBuf::from(".bashrc"));
  assert_eq!(entries[1].relative, PathBuf::from("config/settings.json"));
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_common_directory_as_common_profile() {
  let dir = temp_repo();
  let common = dir.join("common");
  fs::create_dir_all(&common).unwrap();
  fs::write(common.join(".bashrc"), "common").unwrap();
  fs::write(dir.join("._common"), "mac metadata").unwrap();
  fs::write(common.join("._.bashrc"), "mac metadata").unwrap();

  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].profile, "common");
  assert_eq!(entries[0].relative, PathBuf::from(".bashrc"));
  assert_eq!(entries[0].repo_path, common.join(".bashrc"));
  fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_legacy_profiles_common_as_common_profile() {
  let dir = temp_repo();
  let common = dir.join("profiles/common");
  fs::create_dir_all(&common).unwrap();
  fs::write(common.join(".bashrc"), "common").unwrap();

  let entries = scatildr_repo(&dir).unwrap();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].profile, "common");
  assert_eq!(entries[0].relative, PathBuf::from(".bashrc"));
  assert_eq!(entries[0].repo_path, common.join(".bashrc"));
  fs::remove_dir_all(&dir).ok();
}
