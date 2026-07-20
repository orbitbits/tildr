use anyhow::{Result, bail};
use std::fs;
use tildr_core::{
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
  let target = pick::target(
    ctx,
    args.target,
    true,
    Some("Select a file\n-------------\n"),
    PickMode::Managed,
  )?;

  let path = if target.is_absolute() && target.exists() && !target.starts_with(&ctx.home_path) {
    target
  } else {
    match resolve_target(
      ctx,
      Some(target.display().to_string()),
      args.profile.as_deref(),
    )? {
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
