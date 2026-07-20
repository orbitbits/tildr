use anyhow::{Result, bail};
use std::fs;
use tildr_core::{
  config::Config,
  context::Context,
  pick::{self, PickMode},
};
use tildr_utils::pager::page_string;

use crate::utils::target::{ResolvedTarget, resolve_target};

pub struct CatArgs {
  pub target: Option<String>,
  pub less: bool,
  pub profile: Option<String>,
}

pub fn run(ctx: &Context, args: CatArgs) -> Result<()> {
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

  let content = fs::read_to_string(&path)?;

  if args.less {
    page_string(&content)?;
  } else {
    print!("{}", content);
  }

  Ok(())
}
