use super::RestoreArgs;
use crate::utils::{
  auto_commit::auto_commit_dry_run,
  executor::execute_entries,
  target::{ResolvedTarget, collect_resolved_entries, resolve_targets, scan_effective_entries},
};
use anyhow::{Result, bail};
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};
use tildr_repo::ManagedEntry;
use tildr_ui::output::{SummaryKind, print_actions, print_summary};

use tildr_utils::confirm::confirm;
use tildr_utils::ops::{ManagedPathOp, cleanup_empty_ancestors};

pub fn run(ctx: &Context, targets: Vec<String>, args: RestoreArgs) -> Result<()> {
  if args.all {
    return run_all(ctx, args);
  }

  if targets.is_empty() {
    let picked = pick::target(
      ctx,
      None,
      true,
      Some("Select a file\n-------------\n"),
      PickMode::Managed,
    )?;
    return run(ctx, vec![picked.to_string_lossy().into_owned()], args);
  }

  let resolved_targets = resolve_targets(ctx, &targets, args.profile.as_deref())?;
  let commit_target = commit_label(&resolved_targets);
  let entries = collect_entries(&resolved_targets, &args)?;

  run_entries(ctx, entries, &args, &commit_target)
}

fn run_all(ctx: &Context, args: RestoreArgs) -> Result<()> {
  let entries = scan_effective_entries(ctx)?;

  if entries.is_empty() {
    bail!("No managed files found");
  }

  confirm(
    args.force,
    "Restore (restore to HOME) all files from the repository recursively? [y/N]:",
  )?;

  let (restore, skipped, actions) = execute_restore_entries(ctx, entries, args.dry_run)?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Restore { restore, skipped },
    args.dry_run,
    args.quiet,
  );

  auto_commit_dry_run(ctx, "restore all", args.dry_run);

  Ok(())
}

fn run_entries(
  ctx: &Context,
  entries: Vec<ManagedEntry>,
  args: &RestoreArgs,
  commit_target: &str,
) -> Result<()> {
  let (restore, skipped, actions) = execute_restore_entries(ctx, entries, args.dry_run)?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Restore { restore, skipped },
    args.dry_run,
    args.quiet,
  );

  auto_commit_dry_run(ctx, &format!("restore {commit_target}"), args.dry_run);

  Ok(())
}

fn execute_restore_entries(
  ctx: &Context,
  entries: Vec<tildr_repo::ManagedEntry>,
  dry_run: bool,
) -> Result<(usize, usize, Vec<tildr_ui::output::ActionLog>)> {
  execute_entries(entries, dry_run, "Restored", "Would restore", |entry| {
    let home_path = ctx.home_path.join(&entry.relative);
    let repo_relative = entry.repo_path.strip_prefix(&ctx.repo_path)?.to_path_buf();

    if !entry.repo_path.exists() {
      return Ok(false);
    }

    ManagedPathOp::new(&home_path, &entry.repo_path, &entry.relative).restore()?;

    cleanup_empty_ancestors(&ctx.repo_path, &repo_relative);

    Ok(true)
  })
}

fn collect_entries(
  resolved_targets: &[ResolvedTarget],
  args: &RestoreArgs,
) -> Result<Vec<ManagedEntry>> {
  collect_resolved_entries(resolved_targets, |input| {
    confirm(
      args.force,
      &format!("Restore (restore to HOME) all managed files under {input}/? [y/N]:"),
    )
  })
}

fn commit_label(resolved_targets: &[ResolvedTarget]) -> String {
  if resolved_targets.len() == 1 {
    return match &resolved_targets[0] {
      ResolvedTarget::Interactive => "interactive".to_string(),
      ResolvedTarget::File(entry) => entry.relative.display().to_string(),
      ResolvedTarget::Dir { input, .. } => input.clone(),
    };
  }

  format!("{} targets", resolved_targets.len())
}
