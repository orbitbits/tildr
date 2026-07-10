use anyhow::{Result, bail};
use std::{fs, path::PathBuf};
use tildr_core::{
  constants::APP_NAME,
  context::Context,
  pick::{self, PickMode},
};
use tildr_fs::{
  symlink::{create_symlink, is_symlink},
  utils::remove_file_or_dir,
};
use tildr_git::GitIntegration;
use tildr_ui::{
  color::Colorize,
  icons,
  output::{ActionLog, SummaryKind, print_actions, print_summary},
  prompt::MinimalTheme,
};

pub struct MvArgs {
  pub source: Option<String>,
  pub dest: Option<String>,
  pub dry_run: bool,
  pub quiet: bool,
}

pub fn run(ctx: &Context, args: MvArgs) -> Result<()> {
  // --- Resolve source ---
  let source_rel = match args.source {
    Some(ref s) => PathBuf::from(s),
    None => {
      // Interactive: open picker to select the file
      let picked = pick::target(
        ctx,
        None,
        true,
        Some("Select a file\n-------------\n"),
        PickMode::Managed,
      )?;
      // pick::target returns repo_path-based path; get relative
      picked
        .strip_prefix(&ctx.repo_path)
        .unwrap_or(&picked)
        .to_path_buf()
    }
  };

  let source_repo = ctx.repo_path.join(&source_rel);

  if !source_repo.exists() {
    bail!("File is not managed by tildr: {}", source_rel.display());
  }

  // --- Resolve dest ---
  let dest_input = match args.dest {
    Some(ref d) => d.clone(),
    None => {
      // Interactive: prompt for new path
      let title = format!(
        "{} {}\n--------------\n",
        "File selected:".cyan(),
        source_rel.display()
      );
      let legend = format!("\n{} {}\n", "Actions:".bold(), "ctrl+c: cancel".magenta());
      println!("{}", legend);
      println!("{}", title);
      let input: String = dialoguer::Input::with_theme(&MinimalTheme)
        .with_prompt("New (path | name)".bold())
        .interact_text()?;
      input
    }
  };

  // If the user typed only a filename (no directory separator),
  // keep the original directory.
  let dest_rel = {
    let dest_path = PathBuf::from(&dest_input);
    if dest_path.components().count() == 1 {
      // Only a filename — preserve original directory
      match source_rel.parent() {
        Some(parent) if parent != std::path::Path::new("") => parent.join(&dest_path),
        _ => dest_path,
      }
    } else {
      dest_path
    }
  };

  let dest_repo = ctx.repo_path.join(&dest_rel);
  let source_home = ctx.home_path.join(&source_rel);
  let dest_home = ctx.home_path.join(&dest_rel);

  // --- Validations ---
  if source_rel == dest_rel {
    bail!(
      "Source and destination are the same: {}",
      source_rel.display()
    );
  }

  if dest_repo.exists() {
    bail!("Destination already exists in repo: {}", dest_rel.display());
  }

  // --- Dry run ---
  if args.dry_run {
    let actions = vec![ActionLog {
      action: "Would move".to_string(),
      file: format!("{} → {}", source_rel.display(), dest_rel.display()),
    }];
    print_actions(&actions, args.quiet);
    print_summary(
      SummaryKind::Move {
        moved: 1,
        skipped: 0,
      },
      true,
      args.quiet,
    );
    return Ok(());
  }

  // --- Create parent dirs in repo if needed ---
  if let Some(parent) = dest_repo.parent() {
    fs::create_dir_all(parent)?;
  }

  // --- Move file in repo ---
  fs::rename(&source_repo, &dest_repo)?;

  // --- Remove old symlink in HOME ---
  if is_symlink(&source_home) {
    remove_file_or_dir(&source_home)?;
  }

  // --- Create parent dirs in HOME if needed ---
  if let Some(parent) = dest_home.parent() {
    fs::create_dir_all(parent)?;
  }

  // --- Create new symlink in HOME ---
  create_symlink(&dest_repo, &dest_home)?;

  // --- Output ---
  let actions = vec![ActionLog {
    action: format!("{}Moved", icons().check).green(),
    file: format!("{} → {}", source_rel.display(), dest_rel.display()),
  }];
  print_actions(&actions, args.quiet);
  print_summary(
    SummaryKind::Move {
      moved: 1,
      skipped: 0,
    },
    false,
    args.quiet,
  );

  // --- Auto commit ---
  auto_commit(
    ctx,
    &format!("mv {} {}", source_rel.display(), dest_rel.display()),
    args.dry_run,
  );

  Ok(())
}

fn auto_commit(ctx: &Context, msg: &str, dry_run: bool) {
  if ctx.config.git.auto_commit_enabled() && !dry_run {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}
