use anyhow::Result;
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_fs::symlink::{is_symlink, is_symlink_to};
use tildr_repo::scatildr_repo;
use tildr_ui::{color::Colorize, symbols::icons};

#[derive(Debug, serde::Serialize)]
pub struct FileStatus {
  pub path: String,
  pub status: String,
}

pub struct StatusArgs {
  pub json: bool,
  pub counter: bool,
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

  let mut statuses: Vec<FileStatus> = Vec::new();

  for entry in &entries {
    let home_path = ctx.home_path.join(&entry.relative);
    let repo_path = &entry.repo_path;

    let status = if is_symlink(&home_path) {
      if is_symlink_to(&home_path, repo_path) {
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
      path: entry.relative.display().to_string(),
      status: status.to_string(),
    });
  }

  // --- JSON ---
  if args.json {
    println!("{}", serde_json::to_string_pretty(&statuses)?);
    return Ok(());
  }

  let result = counter_all(&statuses)?;

  // --- COUNTER ---
  if args.counter {
    println!("Managed: {}", result.0);
    println!("Linked: {}", result.1[0]);
    println!("Missing: {}", result.1[1]);
    println!("Broken: {}", result.1[2]);
    println!("Not symlink: {}", result.1[3]);

    return Ok(());
  }

  // --- default ---
  let max_len = statuses.iter().map(|s| s.path.len()).max().unwrap_or(0);

  println!();
  println!("{:<width$}  STATUS", "FILE", width = max_len + 2);

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

    println!(
      "{}{:<width$}  {}",
      symbol,
      s.path,
      label,
      width = max_len + 2
    );
  }
  if result.1[1] > 0 || result.1[2] > 0 || result.1[3] > 0 {
    println!(
      "\n-------------------------------\n{}: {} apply",
      "run".cyan(),
      APP_NAME
    );
  }

  Ok(())
}

fn counter_all(statuses: &Vec<FileStatus>) -> Result<(usize, Vec<i32>)> {
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
