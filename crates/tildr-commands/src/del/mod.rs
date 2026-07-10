pub mod run;
use anyhow::Result;
use tildr_core::context::Context;

pub struct DelArgs {
  pub all: bool,
  pub dry_run: bool,
  pub quiet: bool,
  pub force: bool,
  pub purge: bool,
}

pub fn run(ctx: &Context, target: Option<String>, args: DelArgs) -> Result<()> {
  run::run(ctx, target, args)
}
