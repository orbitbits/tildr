use crate::apply::*;
use std::fs;
use std::path::{Path, PathBuf};
use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_fs::symlink::is_symlink_to;

fn test_dir(name: &str) -> PathBuf {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  std::env::temp_dir().join(format!("tildr-test-apply-{name}-{nanos}"))
}

fn setup_context(home: &Path, repo: &Path) -> Context {
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  Context {
    home_path: home.to_path_buf(),
    repo_path: repo.to_path_buf(),
    config,
  }
}

#[test]
fn apply_no_repo_returns_ok() {
  let ctx = setup_context(Path::new("/tmp"), Path::new("/tmp/nonexistent-repo"));
  let result = run(
    &ctx,
    ApplyArgs {
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  );
  assert!(result.is_ok());
}

#[test]
fn apply_dry_run_creates_nothing() {
  let root = test_dir("dry");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  fs::write(repo.join("file.txt"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      dry_run: true,
      force: false,
      verbose: false,
      quiet: false,
    },
  )
  .unwrap();

  assert!(!home.join("file.txt").exists());
  assert!(home.join("file.txt").symlink_metadata().is_err());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_creates_symlink_for_new_file() {
  let root = test_dir("create");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  fs::write(repo.join("file.txt"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(&link, &repo.join("file.txt")));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_update_broken_symlink() {
  let root = test_dir("update");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  fs::write(repo.join("file.txt"), "content").unwrap();
  let wrong = root.join("wrong.txt");
  fs::write(&wrong, "wrong").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(&wrong, home.join("file.txt")).unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(&link, &repo.join("file.txt")));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_skips_regular_file_without_force() {
  let root = test_dir("skip");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  fs::write(repo.join("file.txt"), "new content").unwrap();
  fs::write(home.join("file.txt"), "existing content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let home_file = home.join("file.txt");
  assert!(home_file.is_file());
  assert!(!home_file.is_symlink());
  assert_eq!(fs::read_to_string(&home_file).unwrap(), "existing content");
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_force_replaces_regular_file() {
  let root = test_dir("force");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  fs::write(repo.join("file.txt"), "new content").unwrap();
  fs::write(home.join("file.txt"), "existing").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      dry_run: false,
      force: true,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(&link, &repo.join("file.txt")));
  fs::remove_dir_all(&root).ok();
}
