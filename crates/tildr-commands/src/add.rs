use anyhow::{Result, bail};
use std::{
  fs,
  path::{Path, PathBuf},
};
use tildr_core::{
  context::Context,
  pick::{PickMode, target as pick_target},
};
use tildr_fs::{
  paths::resolve_home_path,
  symlink::{create_symlink, is_symlink, is_symlink_to},
  utils::remove_file_or_dir,
};
use tildr_repo::ignore::IgnoreMatcher;
use tildr_ui::{
  color::Colorize,
  icons,
  output::{ActionLog, SummaryKind, print_actions, print_summary},
};
use tildr_utils::fs::move_file;
use walkdir::WalkDir;

use crate::profile::{
  COMMON_PROFILE, DEFAULT_PROFILE, Profiles, normalize_profile_name, profile_dir,
  validate_profile_component,
};
use crate::utils::{auto_commit::auto_commit_dry_run, tildrignore};

pub struct AddArgs {
  pub paths: Option<Vec<String>>,
  pub profile: Option<String>,
  pub dry_run: bool,
  pub quiet: bool,
  pub force: bool,
  pub nolink: bool,
}

fn resolve_profile(ctx: &Context, profile: &Option<String>) -> Result<String> {
  if let Some(name) = profile {
    let name = normalize_profile_name(name);
    validate_profile_component(name)?;
    if name == COMMON_PROFILE || name == DEFAULT_PROFILE {
      return Ok(name.to_string());
    }
    let profiles = Profiles::load(ctx)?;
    if !profiles.profiles.contains_key(name) {
      bail!("Profile '{}' not found.", name);
    }
    return Ok(name.to_string());
  }
  // Use the active profile, or shared no-profile files when no profile is active.
  let profiles = Profiles::load(ctx)?;
  Ok(
    profiles
      .active
      .clone()
      .unwrap_or_else(|| COMMON_PROFILE.to_string()),
  )
}

pub fn run(ctx: &Context, args: AddArgs) -> Result<()> {
  let profile_name = resolve_profile(ctx, &args.profile)?;

  let resolved_paths = match &args.paths {
    Some(paths) if !paths.is_empty() => paths.clone(),
    _ => {
      // No paths provided — open interactive pick from HOME
      let picked = pick_target(
        ctx,
        None,
        true,
        Some("Add file\n--------\n"),
        PickMode::Home,
      )?;
      let relative = picked
        .strip_prefix(&ctx.home_path)
        .unwrap_or(&picked)
        .display()
        .to_string();
      vec![relative]
    }
  };

  let targets = resolve_add_targets(ctx, &resolved_paths)?;
  let ignore = IgnoreMatcher::from_repo(&ctx.repo_path)?;

  let mut actions = Vec::new();
  let mut added = 0;
  let mut skipped = 0;

  for target in &targets {
    let outcome = if target.source.is_dir() {
      run_dir(ctx, &target.source, &ignore, &args, &profile_name)?
    } else {
      run_file(ctx, &target.source, &ignore, &args, &profile_name)?
    };

    actions.extend(outcome.actions);
    added += outcome.added;
    skipped += outcome.skipped;
  }

  print_actions(&actions, args.quiet);

  print_summary(
    SummaryKind::Add { added, skipped },
    args.dry_run,
    args.quiet,
  );

  if added > 0 && !args.dry_run && !args.nolink {
    crate::apply::run(
      ctx,
      crate::apply::ApplyArgs {
        check: false,
        dry_run: false,
        force: false,
        verbose: false,
        quiet: true,
      },
    )?;
  }

  auto_commit_dry_run(
    ctx,
    &format!("add {} to profile '{profile_name}'", commit_label(&targets)),
    args.dry_run,
  );

  Ok(())
}

fn run_file(
  ctx: &Context,
  source: &Path,
  ignore: &IgnoreMatcher,
  args: &AddArgs,
  profile_name: &str,
) -> Result<AddOutcome> {
  let (did_add, action) = process_add_file(ctx, source, ignore, args, profile_name)?;
  let mut actions = Vec::new();

  if let Some(action) = action {
    actions.push(action);
  }

  Ok(AddOutcome {
    added: usize::from(did_add),
    skipped: usize::from(!did_add),
    actions,
  })
}

fn run_dir(
  ctx: &Context,
  root: &Path,
  ignore: &IgnoreMatcher,
  args: &AddArgs,
  profile_name: &str,
) -> Result<AddOutcome> {
  let mut outcome = AddOutcome::default();
  let mut added = 0;
  let mut skipped = 0;

  for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
    let source = entry.path();

    let relative = to_relative(ctx, source)?;

    if ignore.is_ignored(&relative) {
      continue;
    }

    if !entry.file_type().is_file() {
      continue;
    }

    let (did_add, action) = process_add_file(ctx, source, ignore, args, profile_name)?;

    if did_add {
      added += 1;
    } else {
      skipped += 1;
    }

    if let Some(a) = action {
      outcome.actions.push(a);
    }
  }

  outcome.added = added;
  outcome.skipped = skipped;

  Ok(outcome)
}

fn process_add_file(
  ctx: &Context,
  source: &Path,
  ignore: &IgnoreMatcher,
  args: &AddArgs,
  profile_name: &str,
) -> Result<(bool, Option<ActionLog>)> {
  let relative = to_relative(ctx, source)?;
  let target = profile_dir(&ctx.repo_path, profile_name).join(&relative);
  let target_exists = target.symlink_metadata().is_ok();
  let already_linked = is_symlink(source) && is_symlink_to(source, &target);

  // --- IGNORE ---
  if ignore.is_ignored(&relative) {
    return Ok((false, None));
  }

  // --- ALREADY LINKED (non-nolink) ---
  if !args.nolink && already_linked {
    return Ok((false, None));
  }

  if args.nolink && already_linked {
    if !args.dry_run {
      remove_file_or_dir(source)?;
      tildrignore::append_path(&ctx.repo_path, &relative)?;
    }
    return Ok((
      true,
      Some(ActionLog {
        action: if args.dry_run {
          "Would add (nolink)".to_string()
        } else {
          format!("{}Added (nolink)", icons().check).green()
        },
        file: format!("{}/{}", profile_name, relative.display()),
      }),
    ));
  }

  if target_exists && !args.force {
    bail!(
      "File already exists in profile '{}': {}. Use --force to replace it.",
      profile_name,
      relative.display()
    );
  }

  // --- DRY RUN ---
  if args.dry_run {
    let action = if args.nolink {
      "Would add (nolink)"
    } else {
      "Would add"
    };
    return Ok((
      true,
      Some(ActionLog {
        action: action.to_string(),
        file: format!("{}/{}", profile_name, relative.display()),
      }),
    ));
  }

  // --- PREP TARGET ---
  if let Some(parent) = target.parent() {
    fs::create_dir_all(parent)?;
  }

  if target_exists {
    remove_file_or_dir(&target)?;
  }

  // --- NOLINK: move only, no symlink ---
  if args.nolink {
    // If source is a symlink pointing to repo, just remove it (file is already in repo)
    if is_symlink(source) {
      fs::copy(source, &target)?;
      remove_file_or_dir(source)?;
    } else {
      move_file(source, &target)?;
    }
    tildrignore::append_path(&ctx.repo_path, &relative)?;

    return Ok((
      true,
      Some(ActionLog {
        action: format!("{}Added (nolink)", icons().check).green(),
        file: format!("{}/{}", profile_name, relative.display()),
      }),
    ));
  }

  // --- MOVE + LINK ---
  if is_symlink(source) {
    fs::copy(source, &target)?;
    remove_file_or_dir(source)?;
  } else {
    move_file(source, &target)?;
  }
  create_symlink(&target, source)?;

  Ok((
    true,
    Some(ActionLog {
      action: format!("{}Added", icons().check).green(),
      file: format!("{}/{}", profile_name, relative.display()),
    }),
  ))
}

fn to_relative(ctx: &Context, path: &Path) -> Result<PathBuf> {
  Ok(path.strip_prefix(&ctx.home_path)?.to_path_buf())
}

#[derive(Default)]
struct AddOutcome {
  added: usize,
  skipped: usize,
  actions: Vec<ActionLog>,
}

struct AddTarget {
  source: PathBuf,
  relative: PathBuf,
}

fn resolve_add_targets(ctx: &Context, paths: &[String]) -> Result<Vec<AddTarget>> {
  let mut targets = Vec::with_capacity(paths.len());

  for path in paths {
    let source = resolve_home_path(path, &ctx.home_path);

    if !source.exists() {
      bail!("File does not exist: {}", source.display());
    }

    if !source.is_file() && !source.is_dir() {
      bail!("Unsupported path type: {}", source.display());
    }

    targets.push(AddTarget {
      relative: to_relative(ctx, &source)?,
      source,
    });
  }

  Ok(targets)
}

fn commit_label(targets: &[AddTarget]) -> String {
  if targets.len() == 1 {
    return targets[0].relative.display().to_string();
  }

  format!("{} targets", targets.len())
}
