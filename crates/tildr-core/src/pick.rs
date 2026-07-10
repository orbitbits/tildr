use super::{config::Config, context::Context};
use anyhow::{Result, bail};
use crossterm::{
  cursor::{self, position},
  execute, terminal,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::path::PathBuf;
use tildr_fs::paths::resolve_home_path;
use tildr_repo::scatildr_repo;
use tildr_ui::{color::Colorize, prompt::MinimalTheme};
use tildr_utils::sys::has_display;

pub enum PickMode {
  Managed,
  Home, // list files from home for add pick
}

// Note:
// Fix: Ctrl+C (SIGINT) kills the process before Drop runs on Linux/macOS,
// so RAII guards do not work here. A signal handler registered before the
// interactive prompt ensures the terminal is restored (raw mode disabled,
// cursor shown) before the process exits with code 130.
// Suggested by Claude (claude.ai) — Anthropic.
//
// Module level — outside of any function
#[cfg(unix)]
extern "C" fn handle_sigint(_: libc::c_int) {
  let _ = crossterm::terminal::disable_raw_mode();
  let _ = crossterm::execute!(std::io::stderr(), crossterm::cursor::Show);
  std::process::exit(130);
}

enum CursorMove {
  #[allow(dead_code)]
  Up(u16),
  To(u16, u16),
}

fn restore_terminal(cursor_move: CursorMove, clear_type: terminal::ClearType) {
  let _ = terminal::disable_raw_mode();
  match cursor_move {
    CursorMove::Up(n) => {
      let _ = execute!(
        std::io::stderr(),
        cursor::Show,
        cursor::MoveUp(n),
        terminal::Clear(clear_type),
      );
    }
    CursorMove::To(x, y) => {
      let _ = execute!(
        std::io::stderr(),
        cursor::Show,
        cursor::MoveTo(x, y),
        terminal::Clear(clear_type),
      );
    }
  }
}

pub fn target(
  ctx: &Context,
  input: Option<String>,
  interactive: bool,
  message: Option<&str>,
  mode: PickMode,
) -> Result<PathBuf> {
  // --- Config ---
  if let Some(ref t) = input
    && t == "config"
  {
    return Ok(Config::config_path());
  }

  // --- Interactive ---
  if input.is_none() && interactive {
    #[cfg(unix)]
    unsafe {
      libc::signal(
        libc::SIGINT,
        handle_sigint as *const () as libc::sighandler_t,
      );
    }

    // --- PickMode::Home: It doesn't go through Select ---
    if let PickMode::Home = mode {
      let relative = if has_display() {
        let picked = rfd::FileDialog::new()
          .set_directory(&ctx.home_path)
          .pick_file();

        match picked {
          Some(path) => path
            .strip_prefix(&ctx.home_path)
            .map_err(|_| anyhow::anyhow!("Path must be inside HOME directory"))?
            .to_path_buf(),
          None => std::process::exit(130),
        }
      } else {
        // fallback: type the path
        let input: String = dialoguer::Input::with_theme(&MinimalTheme)
          .with_prompt("File path")
          .interact_text()?;
        PathBuf::from(input)
      };

      return Ok(ctx.home_path.join(&relative));
    }

    // --- PickMode::Managed: Select interactive ---
    let entries = scatildr_repo(&ctx.repo_path)?;
    if entries.is_empty() {
      bail!("No managed files found");
    }

    let all_items: Vec<String> = entries
      .iter()
      .map(|e| e.relative.display().to_string())
      .collect();

    // Saves the current position before printing anything
    let (cursor_x, cursor_y) = position().unwrap_or((0, 0));

    let items = if all_items.len() > ctx.config.core.search_threshold {
      loop {
        let legend = format!(
          "\n{} {}\n",
          "Actions:".bold(),
          "enter: skip   text+enter: search   ctrl+c: cancel".magenta()
        );
        println!("{}", legend);
        // Show search input
        let search: String = dialoguer::Input::with_theme(&MinimalTheme)
          .with_prompt("Search file")
          .allow_empty(true)
          .interact_text()?;

        // Clears the search area before displaying Select
        restore_terminal(
          CursorMove::To(cursor_x, cursor_y),
          terminal::ClearType::FromCursorDown,
        );

        if search.is_empty() {
          break all_items;
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(i64, String)> = all_items
          .clone()
          .into_iter()
          .filter_map(|e| matcher.fuzzy_match(&e, &search).map(|s| (s, e)))
          .collect();

        if scored.is_empty() {
          // Go back to the loop
          println!("{}", "\nNo files matched. Try again.".yellow());
          continue;
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        break scored.into_iter().map(|(_, e)| e).collect();
      }
    } else {
      all_items
    };

    let legend = format!(
      "\n{} {}",
      "Actions:".bold(),
      "↑↓: navigate   enter: select   ctrl+c: cancel".magenta()
    );
    println!("{}", legend);
    println!();

    let result = dialoguer::Select::with_theme(&MinimalTheme)
      .with_prompt(message.unwrap_or("Select a file\n-------------\n"))
      .items(&items)
      .default(0)
      .interact_opt();

    restore_terminal(
      CursorMove::To(cursor_x, cursor_y),
      terminal::ClearType::FromCursorDown,
    );

    return match result {
      Ok(Some(selection)) => Ok(PathBuf::from(&items[selection])),
      Ok(None) | Err(_) => std::process::exit(130),
    };
  }

  // --- File normal ---
  let input = input.ok_or_else(|| anyhow::anyhow!("Target required"))?;

  let home_path = resolve_home_path(&input, &ctx.home_path);

  let relative = home_path
    .strip_prefix(&ctx.home_path)
    .map_err(|_| anyhow::anyhow!("Path must be inside HOME directory"))?
    .to_path_buf();

  let repo_file = ctx.repo_path.join(&relative);

  if !repo_file.exists() {
    bail!("File is not managed by tildr: {}", relative.display());
  }

  Ok(repo_file)
}
