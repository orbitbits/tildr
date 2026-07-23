use crate::status::*;
use std::{fs, path::PathBuf};
use tildr_core::{config::Config, context::Context};

fn test_context(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!(
    "tildr-test-status-{name}-{}-{nanos}",
    std::process::id()
  ));
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  (
    root,
    Context {
      config,
      repo_path: repo,
      home_path: home,
    },
  )
}

fn file_status(profile: &str, filepath: &str, status: &str) -> FileStatus {
  FileStatus {
    profile: profile.to_string(),
    filepath: filepath.to_string(),
    status: status.to_string(),
  }
}

#[test]
fn counter_all_empty() {
  let result = counter_all(&Vec::new()).unwrap();
  assert_eq!(result.0, 0);
  assert_eq!(result.1, vec![0, 0, 0, 0]);
}

#[test]
fn counter_all_linked_only() {
  let statuses = vec![
    file_status("default", "a", "linked"),
    file_status("work", "b", "linked"),
  ];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.0, 2);
  assert_eq!(result.1, vec![2, 0, 0, 0]);
}

#[test]
fn counter_all_mixed_statuses() {
  let statuses = vec![
    file_status("default", "a", "linked"),
    file_status("work", "b", "missing_link"),
    file_status("personal", "c", "broken_symlink"),
    file_status("archlinux", "d", "not_a_symlink"),
  ];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.0, 4);
  assert_eq!(result.1, vec![1, 1, 1, 1]);
}

#[test]
fn counter_all_unknown_status_is_ignored() {
  let statuses = vec![file_status("default", "a", "unknown")];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.1, vec![0, 0, 0, 0]);
}

#[test]
fn file_status_serialization() {
  let fs = FileStatus {
    profile: "default".to_string(),
    filepath: "~/.bashrc".to_string(),
    status: "linked".to_string(),
  };
  let json = serde_json::to_string(&fs).unwrap();
  assert!(json.contains("\"profile\":\"default\""));
  assert!(json.contains("\"filepath\":\"~/.bashrc\""));
  assert!(json.contains("\"status\":\"linked\""));
}

#[test]
fn render_clean_status_shows_summary_without_file_rows() {
  let output = render_clean_status(3).unwrap();

  assert!(output.contains("All 3 files linked correctly."));
  assert!(output.contains("tildr list   (to see all tracked files)"));
  assert!(!output.contains("PROFILE"));
  assert!(!output.contains("FILEPATH"));
}

#[test]
fn render_problem_statuses_groups_only_non_linked_files() {
  let statuses = vec![
    file_status("linux", "~/.bashrc", "linked"),
    file_status("no-profile", "~/.zshrc", "missing_link"),
    file_status("linux", "~/.config/nvim/init.lua", "missing_link"),
    file_status("linux", "~/.gitconfig", "not_a_symlink"),
  ];

  let output = render_problem_statuses(&statuses).unwrap();

  assert!(output.contains("missing link (2)"));
  assert!(output.contains("  ~/.zshrc"));
  assert!(output.contains("  ~/.config/nvim/init.lua"));
  assert!(output.contains("not a symlink (1)"));
  assert!(output.contains("  ~/.gitconfig"));
  assert!(output.contains("tildr apply"));
  assert!(!output.contains("~/.bashrc"));
  assert!(!output.contains("linked"));
  assert!(!output.contains("PROFILE"));
}

#[test]
fn render_all_statuses_keeps_full_table_behavior() {
  let statuses = vec![
    file_status("common", "~/.zshrc", "linked"),
    file_status("linux", "~/.bashrc", "missing_link"),
  ];

  let output = render_all_statuses(&statuses).unwrap();

  assert!(output.contains("PROFILE"));
  assert!(output.contains("FILEPATH"));
  assert!(output.contains("STATUS"));
  assert!(output.contains("no profile"));
  assert!(output.contains("~/.zshrc"));
  assert!(output.contains("linked"));
  assert!(output.contains("~/.bashrc"));
  assert!(output.contains("missing link"));
  assert!(output.contains("tildr apply"));
}

#[test]
fn status_default_returns_error_for_problems_but_all_and_counter_do_not() {
  let (root, ctx) = test_context("missing-link");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "repo").unwrap();

  let default = run(
    &ctx,
    StatusArgs {
      json: false,
      counter: false,
      all: false,
      less: false,
      profile: None,
    },
  );
  assert!(default.is_err());

  let all = run(
    &ctx,
    StatusArgs {
      json: false,
      counter: false,
      all: true,
      less: false,
      profile: None,
    },
  );
  assert!(all.is_ok());

  let counter = run(
    &ctx,
    StatusArgs {
      json: false,
      counter: true,
      all: false,
      less: false,
      profile: None,
    },
  );
  assert!(counter.is_ok());

  fs::remove_dir_all(root).ok();
}
