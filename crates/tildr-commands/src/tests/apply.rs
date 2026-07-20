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

fn set_active(ctx: &Context, name: &str) {
  let mut profiles = crate::profile::Profiles::load(ctx).unwrap();
  profiles.active = Some(name.to_string());
  profiles.save(ctx).unwrap();
}

#[test]
fn apply_no_repo_returns_ok() {
  let ctx = setup_context(Path::new("/tmp"), Path::new("/tmp/nonexistent-repo"));
  let result = run(
    &ctx,
    ApplyArgs {
      check: false,
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
  fs::create_dir_all(repo.join("profiles/default")).unwrap();
  fs::write(repo.join("profiles/default/file.txt"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
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
  fs::create_dir_all(repo.join("profiles/default")).unwrap();
  fs::write(repo.join("profiles/default/file.txt"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(
    &link,
    &repo.join("profiles/default/file.txt")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_prefers_active_profile_over_common() {
  let root = test_dir("active-over-common");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::create_dir_all(repo.join("profiles/linux")).unwrap();
  fs::write(repo.join("common/.bashrc"), "common").unwrap();
  fs::write(repo.join("profiles/linux/.bashrc"), "linux").unwrap();
  let ctx = setup_context(&home, &repo);
  set_active(&ctx, "linux");

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join(".bashrc");
  assert!(link.exists());
  assert!(is_symlink_to(&link, &repo.join("profiles/linux/.bashrc")));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_uses_common_when_no_profile_is_active() {
  let root = test_dir("common-no-active");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "common").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join(".bashrc");
  assert!(link.exists());
  assert!(is_symlink_to(&link, &repo.join("common/.bashrc")));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_update_broken_symlink() {
  let root = test_dir("update");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/default")).unwrap();
  fs::write(repo.join("profiles/default/file.txt"), "content").unwrap();
  let wrong = root.join("wrong.txt");
  fs::write(&wrong, "wrong").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(&wrong, home.join("file.txt")).unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(
    &link,
    &repo.join("profiles/default/file.txt")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_skips_regular_file_without_force() {
  let root = test_dir("skip");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/default")).unwrap();
  fs::write(repo.join("profiles/default/file.txt"), "new content").unwrap();
  fs::write(home.join("file.txt"), "existing content").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
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
  fs::create_dir_all(repo.join("profiles/default")).unwrap();
  fs::write(repo.join("profiles/default/file.txt"), "new content").unwrap();
  fs::write(home.join("file.txt"), "existing").unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: true,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  let link = home.join("file.txt");
  assert!(link.exists());
  assert!(is_symlink_to(
    &link,
    &repo.join("profiles/default/file.txt")
  ));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_check_passes_when_links_are_correct() {
  let root = test_dir("check-ok");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(repo.join("common/.bashrc"), home.join(".bashrc")).unwrap();
  let ctx = setup_context(&home, &repo);

  let result = run(
    &ctx,
    ApplyArgs {
      check: true,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  );

  assert!(result.is_ok());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_check_fails_when_link_is_missing() {
  let root = test_dir("check-missing");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "content").unwrap();
  let ctx = setup_context(&home, &repo);

  let result = run(
    &ctx,
    ApplyArgs {
      check: true,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  );

  assert!(result.is_err());
  assert!(home.join(".bashrc").symlink_metadata().is_err());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_check_fails_on_regular_file_without_replacing_it() {
  let root = test_dir("check-conflict");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("common")).unwrap();
  fs::write(repo.join("common/.bashrc"), "repo content").unwrap();
  fs::write(home.join(".bashrc"), "home content").unwrap();
  let ctx = setup_context(&home, &repo);

  let result = run(
    &ctx,
    ApplyArgs {
      check: true,
      dry_run: false,
      force: true,
      verbose: false,
      quiet: true,
    },
  );

  let home_file = home.join(".bashrc");
  assert!(result.is_err());
  assert!(home_file.is_file());
  assert!(!home_file.is_symlink());
  assert_eq!(fs::read_to_string(&home_file).unwrap(), "home content");
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_removes_link_for_inactive_only_file() {
  let root = test_dir("remove-inactive-link");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/linux")).unwrap();
  fs::write(repo.join("profiles/linux/.zshrc"), "linux").unwrap();
  let link = home.join(".zshrc");
  #[cfg(unix)]
  std::os::unix::fs::symlink(repo.join("profiles/linux/.zshrc"), &link).unwrap();
  let ctx = setup_context(&home, &repo);

  run(
    &ctx,
    ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();

  assert!(link.symlink_metadata().is_err());
  assert!(repo.join("profiles/linux/.zshrc").exists());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn apply_check_reports_link_for_inactive_only_file() {
  let root = test_dir("check-inactive-link");
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(repo.join("profiles/linux")).unwrap();
  fs::write(repo.join("profiles/linux/.zshrc"), "linux").unwrap();
  let link = home.join(".zshrc");
  #[cfg(unix)]
  std::os::unix::fs::symlink(repo.join("profiles/linux/.zshrc"), &link).unwrap();
  let ctx = setup_context(&home, &repo);

  let result = run(
    &ctx,
    ApplyArgs {
      check: true,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  );

  assert!(result.is_err());
  assert!(link.symlink_metadata().is_ok());
  fs::remove_dir_all(&root).ok();
}
