use std::process::Command as StdCommand;

use anyhow::{Context as _, Result};
use console::style;
use tildr_core::context::Context;
use tildr_utils::fs::format_size;

pub fn run(ctx: &Context, output: &Option<String>) -> Result<()> {
  let backup_path = match output {
    Some(p) => std::path::PathBuf::from(p),
    None => {
      let home = dirs::home_dir().context("Could not determine home directory")?;
      let today = chrono::Local::now().format("%Y-%m-%d");
      home.join(format!(".dotfiles-backup-{}.tar.gz", today))
    }
  };

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
