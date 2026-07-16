use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn path(repo_path: &Path) -> PathBuf {
  repo_path.join(".tildrignore")
}

pub fn append(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = path(repo_path);
  let pattern = pattern.trim();

  if pattern.is_empty() {
    anyhow::bail!("Pattern cannot be empty");
  }

  let mut content = if ignore_file.exists() {
    fs::read_to_string(&ignore_file).context("Failed to read .tildrignore")?
  } else {
    String::new()
  };

  if content.lines().any(|l| l.trim() == pattern) {
    return Ok(());
  }

  if !content.ends_with('\n') && !content.is_empty() {
    content.push('\n');
  }
  content.push_str(pattern);
  content.push('\n');

  fs::write(&ignore_file, content).context("Failed to write .tildrignore")?;
  Ok(())
}

pub fn append_path(repo_path: &Path, relative: &Path) -> Result<()> {
  let line = relative.display().to_string();
  append(repo_path, &line)
}

pub fn remove(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = path(repo_path);

  if !ignore_file.exists() {
    anyhow::bail!(".tildrignore does not exist");
  }

  let content = fs::read_to_string(&ignore_file).context("Failed to read .tildrignore")?;
  let pattern = pattern.trim();

  let new_content: String = content
    .lines()
    .filter(|l| l.trim() != pattern)
    .collect::<Vec<_>>()
    .join("\n");

  if new_content == content.trim() {
    anyhow::bail!("Pattern '{}' not found in .tildrignore", pattern);
  }

  let final_content = if new_content.is_empty() {
    String::new()
  } else {
    format!("{}\n", new_content)
  };

  fs::write(&ignore_file, final_content).context("Failed to write .tildrignore")?;
  Ok(())
}

pub fn list(repo_path: &Path) -> Result<Vec<String>> {
  let ignore_file = path(repo_path);

  if !ignore_file.exists() {
    return Ok(vec![]);
  }

  let content = fs::read_to_string(&ignore_file).context("Failed to read .tildrignore")?;

  let patterns: Vec<String> = content
    .lines()
    .map(|l| l.trim().to_string())
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .collect();

  Ok(patterns)
}
