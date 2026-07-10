use anyhow::Result;
use std::process::Command;
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_domain::GitMode;

pub struct GitArgs {
  pub mode: GitMode,
}

pub fn run(ctx: &Context, args: GitArgs) -> Result<()> {
  match args.mode {
    GitMode::Status => run_status(ctx),
  }
}

fn run_status(ctx: &Context) -> Result<()> {
  let git_dir = format!("--git-dir={}", ctx.repo_path.join(".git").display());
  let work_tree = format!("--work-tree={}", ctx.repo_path.display());

  // --porcelain: stable and parseable output; empty = clean
  let porcelain = Command::new("git")
    .args([&git_dir, &work_tree, "status", "--porcelain"])
    .output()?;

  let is_clean = porcelain.stdout.is_empty();

  if is_clean {
    let msg = "Nothing to commit, working tree clean.";
    tildr_ui::success(msg);
    return Ok(());
  }

  // There are changes — it displays a warning and then the normal output of git status.
  let msg = format!("Some files below are not tracked by {}\n", APP_NAME);
  tildr_ui::info(&msg);

  Command::new("git")
    .args([&git_dir, &work_tree, "status"])
    .status()?;

  Ok(())
}
