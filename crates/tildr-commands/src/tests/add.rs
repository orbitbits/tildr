use crate::add::{AddArgs, run};
use std::fs;
use std::path::{Path, PathBuf};
use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_fs::symlink::is_symlink_to;

fn test_context(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tildr-test-add-{name}-{nanos}"));
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

fn args(path: &Path, profile: &str) -> AddArgs {
  AddArgs {
    paths: Some(vec![path.display().to_string()]),
    profile: Some(profile.to_string()),
    dry_run: false,
    quiet: true,
    force: false,
    nolink: false,
  }
}

#[test]
fn add_does_not_overwrite_existing_profile_source_without_force() {
  let (root, ctx) = test_context("no-overwrite");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "repo").unwrap();
  fs::write(ctx.home_path.join(".bashrc"), "home").unwrap();

  let result = run(&ctx, args(&ctx.home_path.join(".bashrc"), "no-profile"));

  assert!(result.is_err());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("common/.bashrc")).unwrap(),
    "repo"
  );
  assert_eq!(
    fs::read_to_string(ctx.home_path.join(".bashrc")).unwrap(),
    "home"
  );
  fs::remove_dir_all(&root).ok();
}

#[test]
fn add_copies_managed_symlink_into_new_profile_variant() {
  let (root, ctx) = test_context("copy-symlink-variant");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(
    ctx.repo_path.join("common/.bashrc"),
    ctx.home_path.join(".bashrc"),
  )
  .unwrap();
  let mut profiles = crate::profile::Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), crate::profile::ProfileDef::default());
  profiles.active = Some("linux".to_string());
  profiles.save(&ctx).unwrap();

  run(&ctx, args(&ctx.home_path.join(".bashrc"), "linux")).unwrap();

  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("profiles/linux/.bashrc")).unwrap(),
    "common"
  );
  assert!(ctx.repo_path.join("common/.bashrc").exists());
  assert!(is_symlink_to(
    &ctx.home_path.join(".bashrc"),
    &ctx.repo_path.join("profiles/linux/.bashrc")
  ));
  fs::remove_dir_all(&root).ok();
}
