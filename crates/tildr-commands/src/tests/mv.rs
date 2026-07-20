use std::fs;
use std::path::{Path, PathBuf};

use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_fs::symlink::is_symlink_to;

use crate::mv::{MvArgs, run};

fn test_dir(name: &str) -> PathBuf {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  std::env::temp_dir().join(format!("tildr-test-mv-{name}-{nanos}"))
}

fn setup_context(home: &Path, repo: &Path) -> Context {
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  config.git.auto_commit = false;
  Context {
    home_path: home.to_path_buf(),
    repo_path: repo.to_path_buf(),
    config,
  }
}

fn set_active(ctx: &Context, name: &str) {
  let mut profiles = crate::profile::Profiles::load(ctx).unwrap();
  profiles.active = Some(name.to_string());
  profiles.save(ctx).unwrap();
}

#[test]
fn mv_renames_common_file_from_home_env_path() {
  let root = test_dir("common-home-env");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.yarnrc"), "content").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(repo.join("common/.yarnrc"), home.join(".yarnrc")).unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    MvArgs {
      source: Some("$HOME/.yarnrc".to_string()),
      dest: Some("$HOME/.yarnrc.bak".to_string()),
      dry_run: false,
      quiet: true,
    },
  )
  .unwrap();

  assert!(!repo.join("common/.yarnrc").exists());
  assert!(repo.join("common/.yarnrc.bak").exists());
  assert!(home.join(".yarnrc").symlink_metadata().is_err());
  assert!(is_symlink_to(
    &home.join(".yarnrc.bak"),
    &repo.join("common/.yarnrc.bak")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn mv_renames_common_file_from_tilde_path() {
  let root = test_dir("common-tilde");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.yarnrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    MvArgs {
      source: Some("~/.yarnrc".to_string()),
      dest: Some("~/.yarnrc.bak".to_string()),
      dry_run: false,
      quiet: true,
    },
  )
  .unwrap();

  assert!(!repo.join("common/.yarnrc").exists());
  assert!(repo.join("common/.yarnrc.bak").exists());
  assert!(is_symlink_to(
    &home.join(".yarnrc.bak"),
    &repo.join("common/.yarnrc.bak")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn mv_renames_active_profile_file_from_logical_path() {
  let root = test_dir("profile-logical");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/linux")).unwrap();
  fs::write(repo.join("profiles/linux/.yarnrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);
  set_active(&ctx, "linux");

  run(
    &ctx,
    MvArgs {
      source: Some(".yarnrc".to_string()),
      dest: Some(".yarnrc.bak".to_string()),
      dry_run: false,
      quiet: true,
    },
  )
  .unwrap();

  assert!(!repo.join("profiles/linux/.yarnrc").exists());
  assert!(repo.join("profiles/linux/.yarnrc.bak").exists());
  assert!(is_symlink_to(
    &home.join(".yarnrc.bak"),
    &repo.join("profiles/linux/.yarnrc.bak")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn mv_preserves_profile_storage_root_when_moving_between_dirs() {
  let root = test_dir("profile-dir-move");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/linux/.config/yarn")).unwrap();
  fs::write(repo.join("profiles/linux/.config/yarn/config"), "content").unwrap();
  let ctx = setup_context(&home, &repo);
  set_active(&ctx, "linux");

  run(
    &ctx,
    MvArgs {
      source: Some(".config/yarn/config".to_string()),
      dest: Some(".config/yarn/config.bak".to_string()),
      dry_run: false,
      quiet: true,
    },
  )
  .unwrap();

  assert!(!repo.join("profiles/linux/.config/yarn/config").exists());
  assert!(repo.join("profiles/linux/.config/yarn/config.bak").exists());
  assert!(!repo.join("profiles/linux/.config/yarn/.config").exists());
  fs::remove_dir_all(&root).ok();
}
