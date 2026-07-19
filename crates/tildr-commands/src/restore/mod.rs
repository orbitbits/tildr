pub mod run;

use anyhow::Result;
use tildr_core::context::Context;

pub struct RestoreArgs {
  pub profile: Option<String>,
  pub all: bool,
  pub dry_run: bool,
  pub quiet: bool,
  pub force: bool,
}

pub fn run(ctx: &Context, targets: Vec<String>, args: RestoreArgs) -> Result<()> {
  run::run(ctx, targets, args)
}
