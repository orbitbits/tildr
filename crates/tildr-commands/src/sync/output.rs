use std::process::Output;
use tildr_core::context::Context;
use tildr_ui::{error, info, success};

use super::git::TrackingBranch;
use super::scenario::{SyncScenario, plural};

pub(super) fn print_dry_run(
  ctx: &Context,
  tracking: &TrackingBranch,
  scenario: SyncScenario,
  would_auto_commit: bool,
  quiet: bool,
) {
  if quiet {
    return;
  }

  if would_auto_commit {
    info("Would commit local changes before syncing");
  }

  match scenario {
    SyncScenario::UpToDate => success("Already up to date"),
    SyncScenario::PushOnly { local_ahead } => info(&format!(
      "Would push {} commit{} to {}/{}",
      local_ahead,
      plural(local_ahead),
      tracking.remote,
      tracking.remote_branch
    )),
    SyncScenario::PullOnly { remote_ahead } => info(&format!(
      "Would pull {} commit{} from {}/{}",
      remote_ahead,
      plural(remote_ahead),
      tracking.remote,
      tracking.remote_branch
    )),
    SyncScenario::Diverged {
      local_ahead,
      remote_ahead,
    } => {
      info(&format!(
        "Would pull {} commit{} from {}/{}",
        remote_ahead,
        plural(remote_ahead),
        tracking.remote,
        tracking.remote_branch
      ));
      info(&format!(
        "Would merge local changes with {} local commit{}",
        local_ahead,
        plural(local_ahead)
      ));
      info(&format!(
        "Would inspect conflicts in {} before pushing",
        ctx.config.core.repo
      ));
    }
  }
}

pub(super) fn print_conflict_message(ctx: &Context, files: &[String]) {
  error("Sync failed - conflicting files:");
  for file in files {
    println!("{file}");
  }
  println!();
  info(&format!(
    "Resolve conflicts manually in {}, then run tildr sync again.",
    ctx.config.core.repo
  ));
}

pub(super) fn command_failure(label: &str, output: &Output) -> String {
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

  if !stderr.is_empty() {
    format!("{label}: {stderr}")
  } else if !stdout.is_empty() {
    format!("{label}: {stdout}")
  } else {
    label.to_string()
  }
}
