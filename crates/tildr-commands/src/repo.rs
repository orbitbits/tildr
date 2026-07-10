use anyhow::{Result, bail};
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_domain::RepoMode;

pub struct RepoArgs {
  pub mode: RepoMode,
}

pub fn run(ctx: &Context, args: RepoArgs) -> Result<()> {
  match args.mode {
    RepoMode::Path => run_path(ctx),
  }
}

fn run_path(ctx: &Context) -> Result<()> {
  if ctx.repo_path.exists() {
    println!("{}", ctx.repo_path.display());
  } else {
    let msg = format!(
      "Repository not found: {}\n\nInitialize it with:\n  {} init",
      ctx.repo_path.display(),
      APP_NAME
    );
    bail!(msg);
  }
  Ok(())
}
