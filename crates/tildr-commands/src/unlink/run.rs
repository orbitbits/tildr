use anyhow::{Result, bail};
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};
use tildr_fs::symlink::is_symlink;
use tildr_repo::ManagedEntry;
use tildr_ui::{
  output::{SummaryKind, print_actions, print_summary},
  warn,
};

use super::UnlinkArgs;
use crate::utils::{
  executor::execute_entries,
  target::{ResolvedTarget, collect_resolved_entries, resolve_targets, scan_all_entries},
};
use tildr_utils::confirm::confirm;
use tildr_utils::ops::{ManagedPathOp, cleanup_empty_ancestors};

pub fn run(ctx: &Context, targets: Vec<String>, all: bool, args: UnlinkArgs) -> Result<()> {
  if all {
    return run_all(ctx, args);
  }

  if targets.is_empty() {
    let picked = pick::target(
      ctx,
      None,
      true,
      Some("Select a file to unlink:"),
      PickMode::Managed,
    )?;
    return run(
      ctx,
      vec![picked.to_string_lossy().into_owned()],
      false,
      args,
    );
  }

  let resolved_targets = resolve_targets(ctx, &targets, args.profile.as_deref())?;
  let entries = collect_entries(&resolved_targets, &args)?;

  run_entries(ctx, entries, &args)
}

fn run_all(ctx: &Context, args: UnlinkArgs) -> Result<()> {
  let entries = scan_all_entries(ctx)?;

  if entries.is_empty() {
    bail!("No managed files found");
  }

  confirm(
    args.force,
    "Unlink (remove symlinks only) all files from the repository recursively? [y/N]:",
  )?;

  run_entries(ctx, entries, &args)
}

fn run_entries(ctx: &Context, entries: Vec<ManagedEntry>, args: &UnlinkArgs) -> Result<()> {
  let (unlinked, skipped, actions) =
    execute_entries(entries, args.dry_run, "Unlinked", "Would unlink", |entry| {
      unlink_entry(ctx, entry, args)
    })?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Unlink { unlinked, skipped },
    args.dry_run,
    args.quiet,
  );

  Ok(())
}

fn unlink_entry(ctx: &Context, entry: &ManagedEntry, args: &UnlinkArgs) -> Result<bool> {
  let home_path = ctx.home_path.join(&entry.relative);

  if home_path.symlink_metadata().is_err() {
    return Ok(false);
  }

  if !is_symlink(&home_path) {
    if !args.quiet {
      warn(&format!("{} is not a symlink", entry.relative.display()));
    }
    return Ok(false);
  }

  if ManagedPathOp::new(&home_path, &entry.repo_path, &entry.relative).unlink()? {
    cleanup_empty_ancestors(&ctx.home_path, &entry.relative);
    return Ok(true);
  }

  Ok(false)
}

fn collect_entries(
  resolved_targets: &[ResolvedTarget],
  args: &UnlinkArgs,
) -> Result<Vec<ManagedEntry>> {
  collect_resolved_entries(resolved_targets, |input| {
    confirm(
      args.force,
      &format!("Unlink (remove symlinks only) all managed files under {input}/? [y/N]:"),
    )
  })
}
