use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_domain::GroupMode;
use tildr_fs::symlink::is_symlink_to;

fn test_dir(name: &str) -> PathBuf {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  std::env::temp_dir().join(format!("tildr-test-group-{name}-{nanos}"))
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

fn group_files(ctx: &Context, name: &str) -> Vec<String> {
  let data = fs::read_to_string(ctx.repo_path.join(".tildr/groups.json")).unwrap();
  let json: Value = serde_json::from_str(&data).unwrap();
  json["groups"][name]
    .as_array()
    .unwrap()
    .iter()
    .map(|value| value.as_str().unwrap().to_string())
    .collect()
}

fn group_exists(ctx: &Context, name: &str) -> bool {
  let data = fs::read_to_string(ctx.repo_path.join(".tildr/groups.json")).unwrap();
  let json: Value = serde_json::from_str(&data).unwrap();
  json["groups"].get(name).is_some()
}

#[test]
fn group_create_normalizes_common_storage_paths() {
  let root = test_dir("common-path");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.wgetrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "net".to_string(),
      files: vec!["common/.wgetrc".to_string()],
    },
  )
  .unwrap();

  assert_eq!(group_files(&ctx, "net"), vec![".wgetrc"]);
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_create_normalizes_no_profile_alias() {
  let root = test_dir("no-profile-path");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.wgetrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "net".to_string(),
      files: vec!["no-profile/.wgetrc".to_string()],
    },
  )
  .unwrap();

  assert_eq!(group_files(&ctx, "net"), vec![".wgetrc"]);
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_create_expands_profile_storage_directories() {
  let root = test_dir("profile-dir");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/linux/.config/nvim")).unwrap();
  fs::write(
    repo.join("profiles/linux/.config/nvim/init.lua"),
    "vim.opt.number = true",
  )
  .unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "editor".to_string(),
      files: vec!["profiles/linux/.config/nvim".to_string()],
    },
  )
  .unwrap();

  assert_eq!(group_files(&ctx, "editor"), vec![".config/nvim/init.lua"]);
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_apply_uses_context_home_path() {
  let root = test_dir("apply-home");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "shell".to_string(),
      files: vec!["common/.bashrc".to_string()],
    },
  )
  .unwrap();
  crate::group::run(
    &ctx,
    &GroupMode::Apply {
      name: "shell".to_string(),
    },
  )
  .unwrap();

  assert!(is_symlink_to(
    &home.join(".bashrc"),
    &repo.join("common/.bashrc")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_apply_normalizes_legacy_stored_paths() {
  let root = test_dir("legacy-stored-path");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::create_dir_all(repo.join(".tildr")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  fs::write(
    repo.join(".tildr/groups.json"),
    r#"{"groups":{"shell":["common/.bashrc"]}}"#,
  )
  .unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Apply {
      name: "shell".to_string(),
    },
  )
  .unwrap();

  assert!(is_symlink_to(
    &home.join(".bashrc"),
    &repo.join("common/.bashrc")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_unlink_uses_context_home_path() {
  let root = test_dir("unlink-home");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(repo.join("common/.bashrc"), home.join(".bashrc")).unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "shell".to_string(),
      files: vec!["common/.bashrc".to_string()],
    },
  )
  .unwrap();
  crate::group::run(
    &ctx,
    &GroupMode::Unlink {
      name: "shell".to_string(),
    },
  )
  .unwrap();

  assert!(home.join(".bashrc").symlink_metadata().is_err());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn group_rename_moves_group_files_to_new_name() {
  let root = test_dir("rename");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  crate::group::run(
    &ctx,
    &GroupMode::Create {
      name: "shell".to_string(),
      files: vec!["common/.bashrc".to_string()],
    },
  )
  .unwrap();
  crate::group::run(
    &ctx,
    &GroupMode::Rename {
      from: Some("shell".to_string()),
      to: Some("terminal".to_string()),
    },
  )
  .unwrap();

  assert!(!group_exists(&ctx, "shell"));
  assert_eq!(group_files(&ctx, "terminal"), vec![".bashrc"]);
  fs::remove_dir_all(&root).ok();
}
