use anyhow::{Result, bail};
use std::fs;
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};
use tildr_utils::pager::page_string;

use crate::utils::target::{FileResolution, resolve_logical_file};

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
    match resolve_logical_file(ctx, &target, args.profile.as_deref())? {
      FileResolution::Found(entry) => entry.repo_path,
      FileResolution::AmbiguousAcrossProfiles(profiles) => {
        bail!(
          "File '{}' exists in multiple profiles: {}. Use --profile <name>.",
          target.display(),
          profiles.join(", ")
        );
      }
      FileResolution::NotManaged => bail!("Target is not managed: {}", target.display()),
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
