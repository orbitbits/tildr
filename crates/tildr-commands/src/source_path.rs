use anyhow::{Result, bail};
use tildr_core::{constants::APP_NAME, context::Context};

use crate::utils::target::{ResolvedTarget, resolve_target};

pub struct SourcePathArgs {
  pub target: String,
  pub profile: Option<String>,
}

pub fn run(ctx: &Context, args: SourcePathArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    let msg = format!("Repository not initialized. Run `{} init` first.", APP_NAME);
    tildr_ui::warn(&msg);
    return Ok(());
  }

  match resolve_target(ctx, Some(args.target), args.profile.as_deref())? {
    ResolvedTarget::File(entry) => {
      println!("{}", repo_display(ctx, &entry.repo_path));
      Ok(())
    }
    ResolvedTarget::Dir { input, .. } => bail!("Target is a directory: {input}"),
    ResolvedTarget::Interactive => unreachable!(),
  }
}

fn repo_display(ctx: &Context, path: &std::path::Path) -> String {
  path
    .strip_prefix(&ctx.home_path)
    .map(|relative| format!("~/{}", relative.display()))
    .unwrap_or_else(|_| path.display().to_string())
}
