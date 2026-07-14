use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tildr_core::context::Context as Ctx;
use tildr_domain::ExcludeMode;
use tildr_git::GitIntegration;

const APP_NAME: &str = "tildr";

pub fn run(ctx: &Ctx, mode: ExcludeMode) -> Result<()> {
  match mode {
    ExcludeMode::Add { pattern } => {
      add_pattern(&ctx.repo_path, &pattern)?;
      auto_commit(ctx, &format!("exclude add {}", pattern));
    }
    ExcludeMode::Remove { pattern } => {
      remove_pattern(&ctx.repo_path, &pattern)?;
      auto_commit(ctx, &format!("exclude remove {}", pattern));
    }
    ExcludeMode::List => list_patterns(&ctx.repo_path)?,
  }
  Ok(())
}

fn auto_commit(ctx: &Ctx, msg: &str) {
  if ctx.config.git.auto_commit_enabled() {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}

fn tildrignore_path(repo_path: &Path) -> PathBuf {
  repo_path.join(".tildrignore")
}

fn add_pattern(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = tildrignore_path(repo_path);
  let pattern = pattern.trim();

  if pattern.is_empty() {
    anyhow::bail!("Pattern cannot be empty");
  }

  let mut content = if ignore_file.exists() {
    fs::read_to_string(&ignore_file).context("Failed to read .tildr/tildrignore")?
  } else {
    String::new()
  };

  // Check for exact match
  if content.lines().any(|l: &str| l.trim() == pattern) {
    println!("Pattern '{}' already in .tildr/tildrignore", pattern);
    return Ok(());
  }

  // Append
  if !content.ends_with('\n') && !content.is_empty() {
    content.push('\n');
  }
  content.push_str(pattern);
  content.push('\n');

  fs::write(&ignore_file, content).context("Failed to write .tildr/tildrignore")?;
  println!("Added '{}' to .tildr/tildrignore", pattern);
  Ok(())
}

fn remove_pattern(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = tildrignore_path(repo_path);

  if !ignore_file.exists() {
    anyhow::bail!(".tildr/tildrignore does not exist");
  }

  let content = fs::read_to_string(&ignore_file).context("Failed to read .tildr/tildrignore")?;
  let pattern = pattern.trim();

  let new_content: String = content
    .lines()
    .filter(|l| l.trim() != pattern)
    .collect::<Vec<_>>()
    .join("\n");

  if new_content == content.trim() {
    anyhow::bail!("Pattern '{}' not found in .tildr/tildrignore", pattern);
  }

  // Ensure trailing newline if content is non-empty
  let final_content = if new_content.is_empty() {
    String::new()
  } else {
    format!("{}\n", new_content)
  };

  fs::write(&ignore_file, final_content).context("Failed to write .tildr/tildrignore")?;
  println!("Removed '{}' from .tildr/tildrignore", pattern);
  Ok(())
}

fn list_patterns(repo_path: &Path) -> Result<()> {
  let ignore_file = tildrignore_path(repo_path);

  if !ignore_file.exists() {
    println!("No .tildr/tildrignore file found");
    return Ok(());
  }

  let content = fs::read_to_string(&ignore_file).context("Failed to read .tildr/tildrignore")?;

  let patterns: Vec<&str> = content
    .lines()
    .map(|l| l.trim())
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .collect();

  if patterns.is_empty() {
    println!("No patterns in .tildr/tildrignore");
  } else {
    for pattern in &patterns {
      println!("{}", pattern);
    }
  }

  Ok(())
}
