use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
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

fn add_pattern(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = repo_path.join(".tildrignore");
  let pattern = pattern.trim();

  if pattern.is_empty() {
    anyhow::bail!("Pattern cannot be empty");
  }

  let mut content = if ignore_file.exists() {
    fs::read_to_string(&ignore_file).context("Failed to read .tildrignore")?
  } else {
    String::new()
  };

  // Check for exact match
  if content.lines().any(|l| l.trim() == pattern) {
    println!("Pattern '{}' already in .tildrignore", pattern);
    return Ok(());
  }

  // Append
  if !content.ends_with('\n') && !content.is_empty() {
    content.push('\n');
  }
  content.push_str(pattern);
  content.push('\n');

  fs::write(&ignore_file, content).context("Failed to write .tildrignore")?;
  println!("Added '{}' to .tildrignore", pattern);
  Ok(())
}

fn remove_pattern(repo_path: &Path, pattern: &str) -> Result<()> {
  let ignore_file = repo_path.join(".tildrignore");

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

  // Ensure trailing newline if content is non-empty
  let final_content = if new_content.is_empty() {
    String::new()
  } else {
    format!("{}\n", new_content)
  };

  fs::write(&ignore_file, final_content).context("Failed to write .tildrignore")?;
  println!("Removed '{}' from .tildrignore", pattern);
  Ok(())
}

fn list_patterns(repo_path: &Path) -> Result<()> {
  let ignore_file = repo_path.join(".tildrignore");

  if !ignore_file.exists() {
    println!("No .tildrignore file found");
    return Ok(());
  }

  let content = fs::read_to_string(&ignore_file).context("Failed to read .tildrignore")?;

  let patterns: Vec<&str> = content
    .lines()
    .map(|l| l.trim())
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .collect();

  if patterns.is_empty() {
    println!("No patterns in .tildrignore");
  } else {
    for pattern in &patterns {
      println!("{}", pattern);
    }
  }

  Ok(())
}
