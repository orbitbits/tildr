use std::process::Command as StdCommand;

use anyhow::{Context as _, Result};
use console::style;
use tildr_core::context::Context;
use tildr_fs::paths::expand_home;
use tildr_utils::fs::format_size;

pub fn run(ctx: &Context, output: &Option<String>) -> Result<()> {
  let backup_path = match output {
    Some(path) => expand_home(path),
    None => {
      let today = chrono::Local::now().format("%Y-%m-%d");
      ctx
        .home_path
        .join(format!(".dotfiles-backup-{}.tar.gz", today))
    }
  };

  if let Some(parent) = backup_path.parent() {
    std::fs::create_dir_all(parent)
      .with_context(|| format!("Failed to create backup directory: {}", parent.display()))?;
  }

  let status = StdCommand::new("tar")
    .arg("-czf")
    .arg(&backup_path)
    .arg("-C")
    .arg(ctx.repo_path.parent().unwrap_or(&ctx.repo_path))
    .arg(ctx.repo_path.file_name().unwrap_or_default())
    .status()
    .context("Failed to run 'tar'. Is it installed?")?;

  if !status.success() {
    anyhow::bail!("tar command failed with exit code: {}", status);
  }

  let size = std::fs::metadata(&backup_path)
    .map(|m| m.len())
    .unwrap_or(0);

  println!(
    "{} {} ({})",
    style("Backup created:").green().bold(),
    backup_path.display(),
    format_size(size)
  );

  Ok(())
}
