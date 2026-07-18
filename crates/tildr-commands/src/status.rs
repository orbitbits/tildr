use anyhow::Result;
use std::{fmt::Write, fs};
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_repo::scatildr_repo;
use tildr_ui::{color::Colorize, symbols::icons};
use tildr_utils::pager::page_string;

use crate::profile::Profiles;

#[derive(Debug, serde::Serialize)]
pub struct FileStatus {
  pub path: String,
  pub status: String,
}

pub struct StatusArgs {
  pub json: bool,
  pub counter: bool,
  pub less: bool,
}

pub fn run(ctx: &Context, args: StatusArgs) -> Result<()> {
  if !ctx.repo_path.exists() {
    let msg = format!("Repository not initialized. Run `{} init` first.", APP_NAME);
    tildr_ui::warn(&msg);
    return Ok(());
  }

  let entries = scatildr_repo(&ctx.repo_path)?
    .into_iter()
    .filter(|e| !e.relative.extension().map(|e| e == "bak").unwrap_or(false))
    .collect::<Vec<_>>();

  if entries.is_empty() {
    let msg = format!("No managed files. Run `{} add <file>` to start.", APP_NAME);
    tildr_ui::info(&msg);
    return Ok(());
  }

  let profiles = Profiles::load(ctx)?;
  let mut statuses: Vec<FileStatus> = Vec::new();

  for entry in &entries {
    let home_path = ctx.home_path.join(&entry.relative);
    let file_str = entry.relative.display().to_string();
    let expected = profiles.resolve(&ctx.repo_path, &file_str);

    let status = match fs::read_link(&home_path) {
      Ok(target) if target == expected => "linked",
      Ok(_) => "broken_symlink",
      Err(_) if home_path.exists() => "not_a_symlink",
      Err(_) => "missing_link",
    };

    statuses.push(FileStatus {
      path: entry.relative.display().to_string(),
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

  // --- default ---
  let mut buf = String::new();
  let max_len = statuses.iter().map(|s| s.path.len()).max().unwrap_or(0);

  writeln!(buf)?;
  writeln!(buf, "{:<width$}  STATUS", "FILE", width = max_len + 2)?;

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
      "{}{:<width$}  {}",
      symbol,
      s.path,
      label,
      width = max_len + 2
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

pub(crate) fn counter_all(statuses: &Vec<FileStatus>) -> Result<(usize, Vec<i32>)> {
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
