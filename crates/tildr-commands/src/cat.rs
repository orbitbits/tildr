use anyhow::Result;
use std::fs;
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};
use tildr_utils::pager::page_string;

pub struct CatArgs {
  pub target: Option<String>,
  pub less: bool,
}

pub fn run(ctx: &Context, args: CatArgs) -> Result<()> {
  let target = pick::target(
    ctx,
    args.target,
    true,
    Some("Select a file\n-------------\n"),
    PickMode::Managed,
  )?;
  let path = ctx.repo_path.join(target);
  let content = fs::read_to_string(&path)?;

  if args.less {
    page_string(&content)?;
  } else {
    print!("{}", content);
  }

  Ok(())
}
