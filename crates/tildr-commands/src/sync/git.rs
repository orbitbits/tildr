use anyhow::{Result, bail};
use std::path::Path;
use std::process::{Command, Output};
use tildr_core::constants::APP_NAME;

use super::output::command_failure;
use super::scenario::{MergeCheck, parse_conflicted_files, parse_upstream_ref};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TrackingBranch {
  pub branch: String,
  pub remote: String,
  pub remote_branch: String,
}

impl TrackingBranch {
  pub(super) fn upstream_ref(&self) -> String {
    format!("{}/{}", self.remote, self.remote_branch)
  }
}

pub(super) struct RepoGit<'a> {
  repo_path: &'a Path,
}

impl<'a> RepoGit<'a> {
  pub(super) fn new(repo_path: &'a Path) -> Self {
    Self { repo_path }
  }

  pub(super) fn current_branch(&self) -> Result<String> {
    let branch = self.stdout(
      ["rev-parse", "--abbrev-ref", "HEAD"],
      "Failed to detect current git branch",
    )?;

    if branch.is_empty() || branch == "HEAD" {
      bail!("No active branch detected");
    }

    Ok(branch)
  }

  pub(super) fn tracking_branch(&self, branch: &str) -> Result<TrackingBranch> {
    if let Ok(upstream) = self.stdout(
      ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
      "Failed to detect tracking branch",
    ) && let Some((remote, remote_branch)) = parse_upstream_ref(&upstream)
    {
      return Ok(TrackingBranch {
        branch: branch.to_string(),
        remote,
        remote_branch,
      });
    }

    let remote = self.stdout(
      vec!["config".to_string(), format!("branch.{branch}.remote")],
      "No tracking remote configured",
    )?;
    let merge_ref = self.stdout(
      vec!["config".to_string(), format!("branch.{branch}.merge")],
      "No tracking branch configured",
    )?;

    let remote_branch = merge_ref
      .strip_prefix("refs/heads/")
      .unwrap_or(&merge_ref)
      .to_string();

    if remote.is_empty() || remote_branch.is_empty() {
      bail!(
        "No git remote configured.\n\nRun:\n  cd $({} repo)\n  git push -u <remote> <branch>",
        APP_NAME
      );
    }

    Ok(TrackingBranch {
      branch: branch.to_string(),
      remote,
      remote_branch,
    })
  }

  pub(super) fn fetch(&self, remote: &str) -> Result<()> {
    self.check(["fetch", remote], "git fetch failed")
  }

  pub(super) fn count_commits(&self, range: &str) -> Result<usize> {
    let output = self.stdout(
      vec![
        "rev-list".to_string(),
        "--count".to_string(),
        range.to_string(),
      ],
      "Failed to count commits",
    )?;
    Ok(output.parse::<usize>()?)
  }

  pub(super) fn fast_forward_merge(&self, upstream_ref: &str) -> Result<()> {
    self.check(
      vec![
        "merge".to_string(),
        "--ff-only".to_string(),
        upstream_ref.to_string(),
      ],
      "git fast-forward merge failed",
    )
  }

  pub(super) fn simulate_merge(&self, upstream_ref: &str) -> Result<MergeCheck> {
    let output = self.output(vec![
      "merge".to_string(),
      "--no-commit".to_string(),
      "--no-ff".to_string(),
      upstream_ref.to_string(),
    ])?;

    if output.status.success() {
      return Ok(MergeCheck::Clean);
    }

    let conflicted = self.conflicted_files()?;
    if conflicted.is_empty() {
      bail!(command_failure("git merge failed", &output));
    }

    Ok(MergeCheck::Conflicted(conflicted))
  }

  pub(super) fn conflicted_files(&self) -> Result<Vec<String>> {
    let output = self.stdout(
      ["diff", "--name-only", "--diff-filter=U"],
      "Failed to list conflicting files",
    )?;

    Ok(parse_conflicted_files(&output))
  }

  pub(super) fn commit_merge(&self, tracking: &TrackingBranch) -> Result<()> {
    self.check(
      vec![
        "commit".to_string(),
        "-m".to_string(),
        format!("Merge {} into {}", tracking.upstream_ref(), tracking.branch),
      ],
      "git merge commit failed",
    )
  }

  pub(super) fn push(&self, tracking: &TrackingBranch, force: bool) -> Result<()> {
    let mut args = vec![
      "push".to_string(),
      tracking.remote.clone(),
      format!("{}:{}", tracking.branch, tracking.remote_branch),
    ];

    if force {
      args.push("--force-with-lease".to_string());
    }

    self.check(args, "git push failed")
  }

  pub(super) fn merge_abort(&self) -> Result<()> {
    self.check(["merge", "--abort"], "git merge abort failed")
  }

  fn stdout<I, S>(&self, args: I, label: &str) -> Result<String>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    let output = self.output(args)?;
    if !output.status.success() {
      bail!(command_failure(label, &output));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
  }

  fn check<I, S>(&self, args: I, label: &str) -> Result<()>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    let output = self.output(args)?;
    if !output.status.success() {
      bail!(command_failure(label, &output));
    }

    Ok(())
  }

  fn output<I, S>(&self, args: I) -> Result<Output>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    let mut command = Command::new("git");
    command.arg(format!(
      "--git-dir={}",
      self.repo_path.join(".git").display()
    ));
    command.arg(format!("--work-tree={}", self.repo_path.display()));

    for arg in args {
      command.arg(arg.as_ref());
    }

    Ok(command.output()?)
  }
}
