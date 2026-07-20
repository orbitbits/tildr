use anyhow::{Result, bail};
use std::process::Command;
use tildr_core::{
  config::Config,
  context::Context,
  pick::{self, PickMode},
};

use crate::utils::target::{ResolvedTarget, resolve_target};

pub struct EditArgs {
  pub target: Option<String>,
  pub profile: Option<String>,
}

pub fn run(ctx: &Context, args: EditArgs) -> Result<()> {
  let editor = std::env::var("EDITOR")
    .or_else(|_| std::env::var("VISUAL"))
    .unwrap_or_else(|_| "nano".to_string());

  let target = if let Some(target) = args.target {
    target
  } else {
    pick::target(
      ctx,
      None,
      true,
      Some("Select a file\n-------------\n"),
      PickMode::Managed,
    )?
    .display()
    .to_string()
  };
  let path = if target == "config" {
    Config::config_path()
  } else {
    match resolve_target(ctx, Some(target), args.profile.as_deref())? {
      ResolvedTarget::File(entry) => entry.repo_path,
      ResolvedTarget::Dir { input, .. } => bail!("Target is a directory: {input}"),
      ResolvedTarget::Interactive => unreachable!(),
    }
  };

  {
    let status = Command::new(&editor).arg(&path).status()?;

    if !status.success() {
      bail!("Editor '{}' exited with an error", editor);
    }
  }

  Ok(())
}
