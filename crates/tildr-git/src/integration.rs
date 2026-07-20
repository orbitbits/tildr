use anyhow::Result;
use std::{path::PathBuf, process::Command};

pub struct GitIntegration {
  pub repo_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitStatusIssueKind {
  Untracked,
  Uncommitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatusIssue {
  pub kind: GitStatusIssueKind,
  pub path: String,
}

impl GitIntegration {
  pub fn new(repo_path: PathBuf) -> Self {
    Self { repo_path }
  }

  pub fn is_git_repo(&self) -> bool {
    self.repo_path.join(".git").exists()
  }

  pub fn init(&self) -> Result<()> {
    let output = Command::new("git")
      .arg("init")
      .current_dir(&self.repo_path)
      .output()?;
    if !output.status.success() {
      anyhow::bail!(
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }
    Ok(())
  }

  pub fn add_all(&self) -> Result<()> {
    let output = Command::new("git")
      .args(["add", "-A"])
      .current_dir(&self.repo_path)
      .output()?;
    if !output.status.success() {
      anyhow::bail!(
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }
    Ok(())
  }

  pub fn commit(&self, message: &str) -> Result<()> {
    let output = Command::new("git")
      .args(["commit", "-m", message])
      .current_dir(&self.repo_path)
      .output()?;
    if !output.status.success() {
      anyhow::bail!(
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }
    Ok(())
  }

  pub fn auto_commit(&self, message: &str) -> Result<()> {
    if !self.is_git_repo() {
      return Ok(());
    }
    self.add_all()?;
    if !self.has_staged_changes()? {
      return Ok(());
    }
    self.commit(message)?;
    Ok(())
  }

  fn has_staged_changes(&self) -> Result<bool> {
    let output = Command::new("git")
      .args(["diff", "--cached", "--quiet", "--exit-code"])
      .current_dir(&self.repo_path)
      .output()?;

    match output.status.code() {
      Some(0) => Ok(false),
      Some(1) => Ok(true),
      _ => anyhow::bail!(
        "git diff --cached failed: {}",
        String::from_utf8_lossy(&output.stderr)
      ),
    }
  }

  pub fn status_issues(&self) -> Result<Vec<GitStatusIssue>> {
    let output = Command::new("git")
      .args(["status", "--porcelain"])
      .current_dir(&self.repo_path)
      .output()?;

    if !output.status.success() {
      anyhow::bail!(
        "git status failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    Ok(parse_status_issues(&String::from_utf8_lossy(
      &output.stdout,
    )))
  }
}

pub fn detect_git_available() -> bool {
  detect_git_available_with(std::env::consts::OS, |program, executable| {
    Command::new(program)
      .arg(executable)
      .output()
      .map(|output| output.status.success())
  })
}

pub(crate) fn detect_git_available_with<F>(os: &str, mut runner: F) -> bool
where
  F: FnMut(&str, &str) -> std::io::Result<bool>,
{
  let (program, executable) = git_locator_command(os);
  runner(program, executable).unwrap_or(false)
}

pub(crate) fn git_locator_command(os: &str) -> (&'static str, &'static str) {
  if os == "windows" {
    ("where", "git")
  } else {
    ("which", "git")
  }
}

pub fn parse_status_issues(output: &str) -> Vec<GitStatusIssue> {
  output.lines().filter_map(parse_status_line).collect()
}

fn parse_status_line(line: &str) -> Option<GitStatusIssue> {
  let bytes = line.as_bytes();
  if bytes.len() < 3 {
    return None;
  }

  let path = line[3..].trim().to_string();
  if path.is_empty() {
    return None;
  }

  match (bytes[0], bytes[1]) {
    (b'?', b'?') => Some(GitStatusIssue {
      kind: GitStatusIssueKind::Untracked,
      path,
    }),
    (left, right) if left != b' ' || right != b' ' => Some(GitStatusIssue {
      kind: GitStatusIssueKind::Uncommitted,
      path,
    }),
    _ => None,
  }
}
