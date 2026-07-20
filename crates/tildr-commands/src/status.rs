use anyhow::Result;
use std::{collections::HashMap, fmt::Write, path::PathBuf};
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_fs::symlink::{is_symlink, is_symlink_to};
use tildr_ui::{color::Colorize, symbols::icons};
use tildr_utils::pager::page_string;

use crate::profile::{Profiles, display_profile_name, normalize_profile_name};
use crate::utils::target::{ManagedEntryProfile, effective_entries, scan_all_entries_with_profile};

#[derive(Debug, serde::Serialize)]
pub struct FileStatus {
  pub profile: String,
  pub filepath: String,
  pub status: String,
}

pub struct StatusArgs {
  pub json: bool,
  pub counter: bool,
  pub less: bool,
  pub profile: Option<String>,
}

pub fn run(ctx: &Context, args: StatusArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    let msg = format!("Repository not initialized. Run `{} init` first.", APP_NAME);
    tildr_ui::warn(&msg);
    return Ok(());
  }

  let entries = scan_all_entries_with_profile(ctx)?;

  if entries.is_empty() {
    let msg = format!("No managed files. Run `{} add <file>` to start.", APP_NAME);
    tildr_ui::info(&msg);
    return Ok(());
  }

  // Group entries by logical filepath to detect variants
  let mut by_filepath: HashMap<PathBuf, Vec<ManagedEntryProfile>> = HashMap::new();
  for entry in entries {
    by_filepath
      .entry(entry.filepath.clone())
      .or_default()
      .push(entry);
  }

  let profiles = Profiles::load(ctx)?;

  // If --profile is specified, filter to only that profile's files.
  // Otherwise show the effective variant for each logical filepath:
  // active profile -> common -> default -> legacy root.
  let mut entries_to_show: Vec<ManagedEntryProfile> = if let Some(ref profile_name) = args.profile {
    let profile_name = normalize_profile_name(profile_name);
    by_filepath
      .values()
      .flat_map(|v| v.iter().filter(|e| e.profile == profile_name).cloned())
      .collect()
  } else {
    effective_entries(&ctx.repo_path, &profiles, &by_filepath)
  };
  entries_to_show.sort_by(|left, right| left.filepath.cmp(&right.filepath));

  if entries_to_show.is_empty() {
    if args.profile.is_some() {
      tildr_ui::info("No managed files for the specified profile.");
    } else {
      tildr_ui::info(
        "No managed files for the active profile. Use --profile to inspect another profile.",
      );
    }
    return Ok(());
  }

  let mut statuses: Vec<FileStatus> = Vec::new();

  for entry in &entries_to_show {
    let home_path = ctx.home_path.join(&entry.filepath);
    let file_str = entry.filepath.display().to_string();
    let expected = profiles.resolve(&ctx.repo_path, &file_str);

    let status = if is_symlink(&home_path) {
      if is_symlink_to(&home_path, &expected) {
        "linked"
      } else {
        "broken_symlink"
      }
    } else if home_path.exists() {
      "not_a_symlink"
    } else {
      "missing_link"
    };

    statuses.push(FileStatus {
      profile: entry.profile.clone(),
      filepath: home_display(&entry.filepath),
      status: status.to_string(),
    });
  }

  // --- JSON ---
  if args.json {
    if args.less {
      page_string(&serde_json::to_string_pretty(&statuses)?)?;
    } else {
      println!("{}", serde_json::to_string_pretty(&statuses)?);
    }
    return Ok(());
  }

  let result = counter_all(&statuses)?;

  // --- COUNTER ---
  if args.counter {
    let output = format!(
      "Managed: {}\nLinked: {}\nMissing: {}\nBroken: {}\nNot symlink: {}",
      result.0, result.1[0], result.1[1], result.1[2], result.1[3]
    );
    if args.less {
      page_string(&output)?;
    } else {
      println!("{}", output);
    }
    return Ok(());
  }

  // --- TABLE ---
  let mut buf = String::new();

  let profile_width = statuses
    .iter()
    .map(|s| display_profile_name(&s.profile).len())
    .max()
    .unwrap_or(7)
    .max(7);

  let filepath_width = statuses
    .iter()
    .map(|s| s.filepath.len())
    .max()
    .unwrap_or(8)
    .max(8);

  writeln!(
    buf,
    "{:<width_p$}  {:<width_f$}  STATUS",
    "PROFILE",
    "FILEPATH",
    width_p = profile_width,
    width_f = filepath_width
  )?;

  for s in &statuses {
    let (symbol, label) = match s.status.as_str() {
      "linked" => (icons().none, format!("{}linked", icons().check).green()),
      "missing_link" => (icons().none, format!("{}missing link", icons().cross).red()),
      "broken_symlink" => (
        icons().none,
        format!("{}broken symlink", icons().cross).red(),
      ),
      "not_a_symlink" => (
        icons().none,
        format!("{}not a symlink", icons().warn).yellow(),
      ),
      _ => (icons().none, "unknown".to_string()),
    };

    writeln!(
      buf,
      "{:<width_p$}  {:<width_f$}  {}{}",
      display_profile_name(&s.profile),
      s.filepath,
      symbol,
      label,
      width_p = profile_width,
      width_f = filepath_width
    )?;
  }

  if result.1[1] > 0 || result.1[2] > 0 || result.1[3] > 0 {
    writeln!(
      buf,
      "\n-------------------------------\n{}: {} apply",
      "run".cyan(),
      APP_NAME
    )?;
  }

  if args.less {
    page_string(&buf)?;
  } else {
    print!("{}", buf);
  }

  Ok(())
}

fn home_display(path: &std::path::Path) -> String {
  if path.as_os_str().is_empty() {
    "~".to_string()
  } else {
    format!("~/{}", path.display())
  }
}

pub(crate) fn counter_all(statuses: &[FileStatus]) -> Result<(usize, Vec<i32>)> {
  let mut linked = 0;
  let mut missing = 0;
  let mut broken = 0;
  let mut not_symlink = 0;

  for s in statuses {
    match s.status.as_str() {
      "linked" => linked += 1,
      "missing_link" => missing += 1,
      "broken_symlink" => broken += 1,
      "not_a_symlink" => not_symlink += 1,
      _ => {}
    }
  }

  let total = statuses.len();
  Ok((total, vec![linked, missing, broken, not_symlink]))
}
