use crate::restore::{RestoreArgs, run};
use std::fs;
use std::path::PathBuf;
use tildr_core::config::Config;
use tildr_core::context::Context;

fn test_context(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tildr-test-restore-{name}-{nanos}"));
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  config.git.auto_commit = false;
  (
    root,
    Context {
      config,
      repo_path: repo,
      home_path: home,
    },
  )
}

#[test]
fn restore_all_moves_only_effective_profile_variant() {
  let (root, ctx) = test_context("effective-all");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(
    ctx.repo_path.join("profiles/linux/.bashrc"),
    ctx.home_path.join(".bashrc"),
  )
  .unwrap();
  let mut profiles = crate::profile::Profiles::load(&ctx).unwrap();
  profiles.active = Some("linux".to_string());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    Vec::new(),
    RestoreArgs {
      profile: None,
      all: true,
      dry_run: false,
      quiet: true,
      force: true,
    },
  )
  .unwrap();

  assert_eq!(
    fs::read_to_string(ctx.home_path.join(".bashrc")).unwrap(),
    "linux"
  );
  assert!(ctx.repo_path.join("common/.bashrc").exists());
  assert!(!ctx.repo_path.join("profiles/linux/.bashrc").exists());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn restore_cleans_empty_physical_storage_directories() {
  let (root, ctx) = test_context("cleanup-storage");
  fs::create_dir_all(ctx.repo_path.join("common/.config/program")).unwrap();
  fs::write(
    ctx.repo_path.join("common/.config/program/config.toml"),
    "config",
  )
  .unwrap();

  run(
    &ctx,
    vec![".config/program/config.toml".to_string()],
    RestoreArgs {
      profile: None,
      all: false,
      dry_run: false,
      quiet: true,
      force: true,
    },
  )
  .unwrap();

  assert!(ctx.home_path.join(".config/program/config.toml").exists());
  assert!(!ctx.repo_path.join("common/.config").exists());
  fs::remove_dir_all(&root).ok();
}
