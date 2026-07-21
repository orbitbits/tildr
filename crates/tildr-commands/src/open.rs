use anyhow::Result;
use tildr_core::{context::Context, file_manager};

pub fn run(ctx: &Context) -> Result<()> {
  file_manager::open_directory(&ctx.config, &ctx.repo_path)
}
