use anyhow::{Context as _, Result};
use tildr_core::context::Context;

pub fn run(ctx: &Context) -> Result<()> {
  open::that_detached(&ctx.repo_path).context("Failed to open file manager")?;
  Ok(())
}
