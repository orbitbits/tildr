use anyhow::{Context as _, Result};
use std::process::Command;
use tildr_core::context::Context;

pub fn run(ctx: &Context) -> Result<()> {
  let file_manager = ctx.config.core.file_manager.trim();
  if file_manager.is_empty() {
    open::that_detached(&ctx.repo_path).context("Failed to open file manager")?;
  } else {
    Command::new(file_manager)
      .arg(&ctx.repo_path)
      .spawn()
      .with_context(|| format!("Failed to open file manager: {file_manager}"))?;
  }

  Ok(())
}
