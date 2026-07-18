use std::fs;

use anyhow::Result;
use tildr_core::context::Context;
use tildr_fs::{
  symlink::{create_symlink, is_symlink, is_symlink_to},
  utils::remove_file_or_dir,
};
use tildr_repo::scatildr_repo;
use tildr_ui::{
  output::{ActionLog, SummaryKind, print_actions, print_summary},
  warn,
};

use crate::profile::Profiles;

pub struct ApplyArgs {
  pub dry_run: bool,
  pub force: bool,
  pub verbose: bool,
  pub quiet: bool,
}

pub fn run(ctx: &Context, args: ApplyArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    warn("Repository not initialized. Run `tildr init` first.");
    return Ok(());
  }

  let entries = scatildr_repo(&ctx.repo_path)?;
  let profiles = Profiles::load(ctx)?;

  let mut actions = Vec::new();
  let mut created = 0;
  let mut updated = 0;
  let mut up_to_date = 0;

  for entry in &entries {
    let home = ctx.home_path.join(&entry.relative);
    let file_str = entry.relative.display().to_string();
    let repo = profiles.resolve(&ctx.repo_path, &file_str);

    let exists = home.exists();
    let is_link = is_symlink(&home);

    // --- Case 1: Correct symlink ---
    if is_link && is_symlink_to(&home, &repo) {
      up_to_date += 1;

      if args.verbose && !args.quiet {
        actions.push(ActionLog {
          action: "Unchanged".to_string(),
          file: file_str,
        });
      }

      continue;
    }

    // --- Decide action type ---
    let (action_str, is_update, needs_removal) = if !exists {
      ("Created", false, false)
    } else if is_link {
      // Broken or wrong symlink → FIX automatically
      ("Updated", true, true)
    } else {
      // Regular file / dir
      if args.force {
        ("Updated", true, true)
      } else {
        up_to_date += 1;

        if args.verbose && !args.quiet {
          actions.push(ActionLog {
            action: "Skipped".to_string(),
            file: file_str,
          });
        }

        continue;
      }
    };

    // --- Dry run ---
    if args.dry_run {
      let action = if is_update {
        "Would update"
      } else {
        "Would create"
      };

      actions.push(ActionLog {
        action: action.to_string(),
        file: file_str,
      });

      if is_update {
        updated += 1;
      } else {
        created += 1;
      }

      continue;
    }

    // --- Apply changes ---
    if needs_removal {
      let _ = remove_file_or_dir(&home);
    }

    if let Some(parent) = home.parent() {
      fs::create_dir_all(parent)?;
    }

    create_symlink(&repo, &home)?;

    actions.push(ActionLog {
      action: action_str.to_string(),
      file: file_str,
    });

    if is_update {
      updated += 1;
    } else {
      created += 1;
    }
  }

  print_actions(&actions, args.quiet);

  print_summary(
    SummaryKind::Apply {
      created,
      updated,
      up_to_date,
    },
    args.dry_run,
    args.quiet,
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
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
    assert!(!home.join("file.txt").symlink_metadata().is_ok());
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
    // Create a symlink pointing to a wrong target
    let wrong = root.join("wrong.txt");
    fs::write(&wrong, "wrong").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&wrong, &home.join("file.txt")).unwrap();
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
}
