// I'm grateful to the AI ​​for helping me with most of these tests.
// Suggested by Claude (claude.ai) — Anthropic.

#[cfg(test)]
mod tests {
  use crate::{
    dispatch,
    sync::scenario::{
      SyncScenario, classify_sync_scenario, parse_conflicted_files, parse_upstream_ref,
    },
  };
  use anyhow::{Result, bail};
  use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, MutexGuard, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
  };
  use tildr_core::Config;
  use tildr_domain::Commands;

  static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

  #[test]
  fn sync_pushes_local_commits_to_remote() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("push")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config()?;

    write_file(&fixture.repo, "local.txt", "local change\n")?;
    git(&fixture.repo, ["add", "."])?;
    git(&fixture.repo, ["commit", "-m", "local change"])?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert_eq!(
      head_ref(&fixture.repo, "HEAD")?,
      bare_head_ref(&fixture.remote, &fixture.branch)?
    );

    Ok(())
  }

  #[test]
  fn sync_pulls_remote_commits_cleanly() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("pull")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config()?;

    let other = fixture.root.join("other");
    clone_repo(&fixture.remote, &other)?;
    configure_git_user(&other)?;
    write_file(&other, "remote.txt", "remote change\n")?;
    git(&other, ["add", "."])?;
    git(&other, ["commit", "-m", "remote change"])?;
    git(&other, ["push", "origin", "HEAD"])?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert_eq!(
      head_ref(&fixture.repo, "HEAD")?,
      bare_head_ref(&fixture.remote, &fixture.branch)?
    );
    assert_eq!(
      fs::read_to_string(fixture.repo.join("remote.txt"))?,
      "remote change\n"
    );

    Ok(())
  }

  #[test]
  fn sync_commits_dirty_worktree_before_pushing() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("auto-commit")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config_with(|config| config.git.auto_commit = true)?;

    write_file(&fixture.repo, "dirty.txt", "dirty local change\n")?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert!(git_stdout(&fixture.repo, ["status", "--porcelain"])?.is_empty());
    assert_eq!(
      git_stdout(&fixture.repo, ["log", "-1", "--pretty=%s"])?,
      "tildr: sync local changes"
    );
    assert_eq!(
      head_ref(&fixture.repo, "HEAD")?,
      bare_head_ref(&fixture.remote, &fixture.branch)?
    );

    Ok(())
  }

  #[test]
  fn sync_uses_configured_remote_when_branch_has_no_upstream() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("configured-remote")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config_with(|config| {
      config.git.auto_commit = true;
      config.git.sync_remote = "origin".to_string();
      config.git.sync_branch = fixture.branch.clone();
    })?;
    git(&fixture.repo, ["branch", "--unset-upstream"])?;

    write_file(&fixture.repo, "configured.txt", "configured remote\n")?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert_eq!(
      head_ref(&fixture.repo, "HEAD")?,
      bare_head_ref(&fixture.remote, &fixture.branch)?
    );

    Ok(())
  }

  #[test]
  fn sync_uses_current_branch_when_configured_sync_branch_is_empty() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("configured-remote-current-branch")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config_with(|config| {
      config.git.auto_commit = true;
      config.git.sync_remote = "origin".to_string();
    })?;
    git(&fixture.repo, ["branch", "--unset-upstream"])?;

    write_file(
      &fixture.repo,
      "current-branch.txt",
      "configured remote branch fallback\n",
    )?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert_eq!(
      head_ref(&fixture.repo, "HEAD")?,
      bare_head_ref(&fixture.remote, &fixture.branch)?
    );

    Ok(())
  }

  #[test]
  fn sync_dry_run_does_not_commit_dirty_worktree() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("dry-run-dirty")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config_with(|config| config.git.auto_commit = true)?;
    let head_before = head_ref(&fixture.repo, "HEAD")?;

    write_file(&fixture.repo, "dry-run.txt", "not committed\n")?;

    dispatch(Commands::Sync {
      dry_run: true,
      quiet: true,
      force: false,
    })?;

    assert_eq!(head_ref(&fixture.repo, "HEAD")?, head_before);
    assert!(!git_stdout(&fixture.repo, ["status", "--porcelain"])?.is_empty());

    Ok(())
  }

  #[test]
  fn sync_respects_disabled_auto_commit() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("auto-commit-disabled")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config_with(|config| config.git.auto_commit = false)?;
    let head_before = head_ref(&fixture.repo, "HEAD")?;

    write_file(&fixture.repo, "disabled.txt", "not committed\n")?;

    dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    })?;

    assert_eq!(head_ref(&fixture.repo, "HEAD")?, head_before);
    assert!(!git_stdout(&fixture.repo, ["status", "--porcelain"])?.is_empty());

    Ok(())
  }

  #[test]
  fn classify_sync_scenarios() {
    assert_eq!(classify_sync_scenario(0, 0), SyncScenario::UpToDate);
    assert_eq!(
      classify_sync_scenario(2, 0),
      SyncScenario::PushOnly { local_ahead: 2 }
    );
    assert_eq!(
      classify_sync_scenario(0, 3),
      SyncScenario::PullOnly { remote_ahead: 3 }
    );
    assert_eq!(
      classify_sync_scenario(2, 3),
      SyncScenario::Diverged {
        local_ahead: 2,
        remote_ahead: 3,
      }
    );
  }

  #[test]
  fn parse_upstream_branch_keeps_slashes_in_branch_name() {
    assert_eq!(
      parse_upstream_ref("origin/feature/nested"),
      Some(("origin".to_string(), "feature/nested".to_string()))
    );
  }

  #[test]
  fn parse_conflicted_files_collects_all_paths() {
    assert_eq!(
      parse_conflicted_files("config/.bashrc\nconfig/.vimrc\n"),
      vec!["config/.bashrc".to_string(), "config/.vimrc".to_string()]
    );
  }

  #[test]
  fn sync_aborts_cleanly_when_merge_conflicts_exist() -> Result<()> {
    let _env_lock = lock_env();
    let fixture = SyncFixture::new("conflict")?;
    let _env_guard = EnvGuard::set(&fixture.home, &fixture.xdg_config);

    write_config()?;

    let other = fixture.root.join("other");
    clone_repo(&fixture.remote, &other)?;
    configure_git_user(&other)?;
    write_file(&other, "shared.txt", "remote version\n")?;
    git(&other, ["add", "shared.txt"])?;
    git(&other, ["commit", "-m", "remote conflict"])?;
    git(&other, ["push", "origin", "HEAD"])?;

    write_file(&fixture.repo, "shared.txt", "local version\n")?;
    git(&fixture.repo, ["add", "shared.txt"])?;
    git(&fixture.repo, ["commit", "-m", "local conflict"])?;

    let local_head_before = head_ref(&fixture.repo, "HEAD")?;
    let result = dispatch(Commands::Sync {
      dry_run: false,
      quiet: true,
      force: false,
    });

    assert!(result.is_err());
    assert!(
      result
        .as_ref()
        .err()
        .map(ToString::to_string)
        .is_some_and(|msg| msg.contains("merge conflicts"))
    );
    assert!(!fixture.repo.join(".git").join("MERGE_HEAD").exists());
    assert!(git_stdout(&fixture.repo, ["status", "--porcelain"])?.is_empty());
    assert_eq!(head_ref(&fixture.repo, "HEAD")?, local_head_before);

    Ok(())
  }

  struct SyncFixture {
    root: PathBuf,
    home: PathBuf,
    xdg_config: PathBuf,
    remote: PathBuf,
    repo: PathBuf,
    branch: String,
  }

  impl SyncFixture {
    fn new(name: &str) -> Result<Self> {
      let root = unique_dir(name);
      let home = root.join("home");
      let xdg_config = root.join("xdg");
      let remote = root.join("remote.git");
      let seed = root.join("seed");
      let repo = home.join(".dotfiles");

      fs::create_dir_all(&root)?;
      fs::create_dir_all(&home)?;
      fs::create_dir_all(&xdg_config)?;

      git_in(&root, ["init", "--bare", "remote.git"])?;
      clone_repo(&remote, &seed)?;
      configure_git_user(&seed)?;

      write_file(&seed, "shared.txt", "base\n")?;
      git(&seed, ["add", "."])?;
      git(&seed, ["commit", "-m", "initial"])?;
      git(&seed, ["push", "-u", "origin", "HEAD"])?;

      clone_repo(&remote, &repo)?;
      configure_git_user(&repo)?;
      let branch = git_stdout(&repo, ["rev-parse", "--abbrev-ref", "HEAD"])?;

      Ok(Self {
        root,
        home,
        xdg_config,
        remote,
        repo,
        branch,
      })
    }
  }

  impl Drop for SyncFixture {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.root);
    }
  }

  struct EnvGuard {
    home: Option<OsString>,
    xdg_config_home: Option<OsString>,
  }

  impl EnvGuard {
    fn set(home: &Path, xdg_config_home: &Path) -> Self {
      let previous_home = std::env::var_os("HOME");
      let previous_xdg = std::env::var_os("XDG_CONFIG_HOME");

      unsafe {
        std::env::set_var("HOME", home);
        std::env::set_var("XDG_CONFIG_HOME", xdg_config_home);
      }

      Self {
        home: previous_home,
        xdg_config_home: previous_xdg,
      }
    }
  }

  impl Drop for EnvGuard {
    fn drop(&mut self) {
      unsafe {
        match &self.home {
          Some(value) => std::env::set_var("HOME", value),
          None => std::env::remove_var("HOME"),
        }

        match &self.xdg_config_home {
          Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
          None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
      }
    }
  }

  fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK
      .get_or_init(|| Mutex::new(()))
      .lock()
      .expect("environment lock poisoned")
  }

  fn write_config() -> Result<()> {
    write_config_with(|_| {})
  }

  fn write_config_with(update: impl FnOnce(&mut Config)) -> Result<()> {
    let mut config = Config::default();
    config.core.repo = "~/.dotfiles".to_string();
    config.git.available = true;
    config.git.enable = None;
    config.git.auto_commit = false;
    update(&mut config);
    config.save()
  }

  fn clone_repo(remote: &Path, target: &Path) -> Result<()> {
    let remote = remote.to_string_lossy().to_string();
    let target = target.to_string_lossy().to_string();
    git_in(Path::new("."), ["clone", remote.as_str(), target.as_str()])
  }

  fn configure_git_user(repo: &Path) -> Result<()> {
    git(repo, ["config", "user.name", "Tildr Test"])?;
    git(repo, ["config", "user.email", "tildr@example.com"])
  }

  fn head_ref(repo: &Path, rev: &str) -> Result<String> {
    git_stdout(repo, ["rev-parse", rev])
  }

  fn bare_head_ref(remote: &Path, branch: &str) -> Result<String> {
    let remote_arg = format!("--git-dir={}", remote.display());
    git_stdout_in(Path::new("."), [remote_arg.as_str(), "rev-parse", branch])
  }

  fn write_file(repo: &Path, relative: &str, content: &str) -> Result<()> {
    let path = repo.join(relative);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
  }

  fn git(repo: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    let output = git_output(repo, args)?;
    if !output.status.success() {
      bail!(git_error(&output));
    }
    Ok(())
  }

  fn git_stdout(repo: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<String> {
    let output = git_output(repo, args)?;
    if !output.status.success() {
      bail!(git_error(&output));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
  }

  fn git_in(dir: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    let output = command_output(dir, "git", args)?;
    if !output.status.success() {
      bail!(git_error(&output));
    }
    Ok(())
  }

  fn git_stdout_in(dir: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<String> {
    let output = command_output(dir, "git", args)?;
    if !output.status.success() {
      bail!(git_error(&output));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
  }

  fn git_output(
    repo: &Path,
    args: impl IntoIterator<Item = impl AsRef<str>>,
  ) -> Result<std::process::Output> {
    command_output(repo, "git", args)
  }

  fn command_output(
    dir: &Path,
    program: &str,
    args: impl IntoIterator<Item = impl AsRef<str>>,
  ) -> Result<std::process::Output> {
    let mut command = Command::new(program);
    command.current_dir(dir);

    for arg in args {
      command.arg(arg.as_ref());
    }

    Ok(command.output()?)
  }

  fn git_error(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if !stderr.is_empty() { stderr } else { stdout }
  }

  fn unique_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("time went backwards")
      .as_nanos();

    std::env::temp_dir().join(format!("tildr-sync-{name}-{}-{nanos}", std::process::id()))
  }
}
