pub mod run;

use anyhow::Result;
use tildr_core::context::Context;

pub struct UnlinkArgs {
  pub dry_run: bool,
  pub quiet: bool,
  pub force: bool,
}

pub fn run(ctx: &Context, targets: Vec<String>, all: bool, args: UnlinkArgs) -> Result<()> {
  run::run(ctx, targets, all, args)
}
