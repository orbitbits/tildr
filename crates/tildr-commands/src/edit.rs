use anyhow::{Result, bail};
use std::process::Command;
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};

pub struct EditArgs {
  pub target: Option<String>,
}

pub fn run(ctx: &Context, args: EditArgs) -> Result<()> {
  let editor = std::env::var("EDITOR")
    .or_else(|_| std::env::var("VISUAL"))
    .unwrap_or_else(|_| "nano".to_string());

  let target = pick::target(
    ctx,
    args.target,
    true,
    Some("Select a file\n-------------\n"),
    PickMode::Managed,
  )?;
  let path = ctx.repo_path.join(target);

  {
    let status = Command::new(&editor).arg(&path).status()?;

    if !status.success() {
      bail!("Editor '{}' exited with an error", editor);
    }
  }

  Ok(())
}
