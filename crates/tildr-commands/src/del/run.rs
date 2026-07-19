use super::DelArgs;
use crate::utils::{
  auto_commit::auto_commit_dry_run,
  executor::execute_entries,
  target::{ResolvedTarget, resolve_target, scan_all_entries},
};
use anyhow::{Result, bail};
use tildr_core::{
  constants::APP_NAME,
  context::Context,
  pick::{self, PickMode},
};
use tildr_repo::ManagedEntry;
use tildr_ui::{
  color::Colorize,
  icons,
  output::{SummaryKind, print_actions, print_summary},
};

use tildr_utils::confirm::confirm;
use tildr_utils::ops::{DeletionMode, ManagedPathOp, cleanup_empty_ancestors};

pub fn run(ctx: &Context, target: Option<String>, args: DelArgs) -> Result<()> {
  if args.all {
    return run_all(ctx, args);
  }

  if !ctx.repo_path.exists() {
    bail!("Repository not found. Run `{} init`", APP_NAME);
  }

  if args.all && target.is_some() {
    bail!("Cannot use --all with a target");
  }

  match resolve_target(ctx, target, args.profile.as_deref())? {
    ResolvedTarget::Interactive => {
      let picked = pick::target(
        ctx,
        None,
        true,
        Some("Select a file\n-------------\n"),
        PickMode::Managed,
      )?;
      run(ctx, Some(picked.to_string_lossy().into_owned()), args)
    }
    ResolvedTarget::File(entry) => run_file(ctx, entry, &args),
    ResolvedTarget::Dir { input, entries } => run_dir(ctx, &input, entries, &args),
  }
}

fn run_all(ctx: &Context, args: DelArgs) -> Result<()> {
  let entries = scan_all_entries(ctx)?;

  if entries.is_empty() {
    bail!("No managed files found");
  }

  confirm(
    args.force,
    "DELETE all files from the repository recursively (dangerous)? [y/N]:",
  )?;

  let (deleted, skipped, actions) = execute_delete_entries(ctx, entries, args.dry_run, args.purge)?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Delete { deleted, skipped },
    args.dry_run,
    args.quiet,
  );

  auto_commit_dry_run(ctx, "delete all", args.dry_run);

  Ok(())
}

fn run_file(ctx: &Context, entry: ManagedEntry, args: &DelArgs) -> Result<()> {
  let commit_target = entry.relative.display().to_string();
  let (deleted, skipped, actions) =
    execute_delete_entries(ctx, vec![entry], args.dry_run, args.purge)?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Delete { deleted, skipped },
    args.dry_run,
    args.quiet,
  );

  auto_commit_dry_run(ctx, &format!("delete {commit_target}"), args.dry_run);

  Ok(())
}

fn run_dir(ctx: &Context, input: &str, entries: Vec<ManagedEntry>, args: &DelArgs) -> Result<()> {
  confirm(
    args.force,
    &format!("DELETE all managed files under {input}/? [y/N]:"),
  )?;

  let (deleted, skipped, actions) = execute_delete_entries(ctx, entries, args.dry_run, args.purge)?;

  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Delete { deleted, skipped },
    args.dry_run,
    args.quiet,
  );

  auto_commit_dry_run(ctx, &format!("delete {input}"), args.dry_run);

  Ok(())
}

fn labels_choice(status: &str) -> Vec<String> {
  let action_label = format!("{}Delete ({})", icons().warn, status).yellow();
  let dry_label = action_label.clone();

  vec![action_label, dry_label]
}

fn execute_delete_entries(
  ctx: &Context,
  entries: Vec<ManagedEntry>,
  dry_run: bool,
  purge: bool,
) -> Result<(usize, usize, Vec<tildr_ui::output::ActionLog>)> {
  let mode = DeletionMode::from(purge);
  let labels = labels_choice(mode.label());

  execute_entries(entries, dry_run, &labels[0], &labels[1], |entry| {
    let home_path = ctx.home_path.join(&entry.relative);

    if !entry.repo_path.exists() {
      return Ok(false);
    }

    ManagedPathOp::new(&home_path, &entry.repo_path, &entry.relative).delete(mode)?;
    cleanup_empty_ancestors(&ctx.repo_path, &entry.relative);

    Ok(true)
  })
}
