use super::{config::Config, context::Context};
use anyhow::{Result, bail};
use crossterm::{
  cursor::{self, position},
  execute, terminal,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::collections::HashMap;
use std::path::PathBuf;
use tildr_fs::paths::{current_dir_under_home, normalize_lexically, resolve_home_path};
use tildr_repo::scatildr_repo;
use tildr_ui::{color::Colorize, prompt::MinimalTheme};
use tildr_utils::sys::has_display;
use walkdir::WalkDir;

fn prompt_home_relative_path(ctx: &Context) -> Result<PathBuf> {
  let input: String = dialoguer::Input::with_theme(&MinimalTheme)
    .with_prompt("File path")
    .interact_text()?;
  Ok(
    resolve_home_path(&input, &ctx.home_path)
      .strip_prefix(&ctx.home_path)
      .map_err(|_| anyhow::anyhow!("Path must be inside HOME directory"))?
      .to_path_buf(),
  )
}

pub enum PickMode {
  Managed,
  Home, // list files from home for add pick
}

fn logical_candidates(ctx: &Context, input: &str, home_path: &std::path::Path) -> Vec<PathBuf> {
  let mut candidates = Vec::new();

  if let Ok(relative) = home_path.strip_prefix(&ctx.home_path) {
    candidates.push(relative.to_path_buf());
  }

  let input_path = std::path::Path::new(input);
  if !input_path.is_absolute()
    && input != "~"
    && !input.starts_with("~/")
    && input != "$HOME"
    && !input.starts_with("$HOME/")
    && let Some(cwd) = current_dir_under_home(&ctx.home_path)
  {
    let cwd_path = normalize_lexically(&cwd.join(input_path));
    if let Ok(relative) = cwd_path.strip_prefix(&ctx.home_path)
      && !candidates.iter().any(|candidate| candidate == relative)
    {
      candidates.push(relative.to_path_buf());
    }
  }

  candidates
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
        prompt_home_relative_path(ctx)?
      };

      return Ok(ctx.home_path.join(&relative));
    }

    // --- PickMode::Managed: Select interactive ---
    let entries = scatildr_repo(&ctx.repo_path)?;

    // Build logical-path → repo-path map, including profile variants.
    let root_set: std::collections::HashSet<PathBuf> =
      entries.iter().map(|e| e.relative.clone()).collect();
    let mut path_map: HashMap<String, PathBuf> = entries
      .iter()
      .map(|e| (e.relative.display().to_string(), e.repo_path.clone()))
      .collect();

    // Maps display string (repo-relative) → logical path for resolution.
    let mut display_to_logical: HashMap<String, String> = HashMap::new();

    let profiles_dir = ctx.repo_path.join("profiles");
    if profiles_dir.is_dir() {
      for profile_entry in std::fs::read_dir(&profiles_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
      {
        let dir = profile_entry.path();
        if !dir.is_dir() {
          continue;
        }
        let profile_name = dir
          .file_name()
          .map(|n| n.to_string_lossy().to_string())
          .unwrap_or_default();
        for variant in WalkDir::new(&dir)
          .into_iter()
          .filter_map(|e| e.ok())
          .filter(|e| e.file_type().is_file())
        {
          let full = variant.path();
          let logical = full.strip_prefix(&dir).unwrap_or(full).to_path_buf();
          let key = logical.display().to_string();
          if !root_set.contains(&logical) && !path_map.contains_key(&key) {
            path_map.insert(key.clone(), full.to_path_buf());
            let display = format!("profiles/{profile_name}/{key}");
            display_to_logical.insert(display, key);
          }
        }
      }
    }

    if path_map.is_empty() {
      bail!("No managed files found");
    }

    // Build display items: use repo-relative paths for profile variants,
    // logical paths for default (root) files.
    let mut display_items: Vec<String> = path_map
      .keys()
      .map(|k| {
        display_to_logical
          .iter()
          .find(|(_, logical)| logical.as_str() == k.as_str())
          .map(|(display, _)| display.clone())
          .unwrap_or_else(|| k.clone())
      })
      .collect();
    display_items.sort();

    // Saves the current position before printing anything
    let (cursor_x, cursor_y) = position().unwrap_or((0, 0));

    let items = if display_items.len() > ctx.config.core.search_threshold {
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
          break display_items;
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(i64, String)> = display_items
          .clone()
          .into_iter()
          .filter_map(|e| matcher.fuzzy_match(&e, &search).map(|s| (s, e)))
          .collect();

        if scored.is_empty() {
          // Go back to the loop
          println!("{}", "\nNo files matched. Try again.".yellow());
          continue;
        }

        scored.sort_by_key(|b| std::cmp::Reverse(b.0));
        break scored.into_iter().map(|(_, e)| e).collect();
      }
    } else {
      display_items
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
      Ok(Some(selection)) => {
        let display = &items[selection];
        // Resolve display string back to logical path for proper resolution.
        let logical = display_to_logical
          .get(display)
          .cloned()
          .unwrap_or_else(|| display.clone());
        Ok(PathBuf::from(logical))
      }
      Ok(None) | Err(_) => std::process::exit(130),
    };
  }

  // --- File normal ---
  let input = input.ok_or_else(|| anyhow::anyhow!("Target required"))?;

  let home_path = resolve_home_path(&input, &ctx.home_path);

  if matches!(mode, PickMode::Managed) {
    let entries = scatildr_repo(&ctx.repo_path)?;

    for relative in logical_candidates(ctx, &input, &home_path) {
      if entries.iter().any(|entry| entry.relative == relative)
        || entries
          .iter()
          .any(|entry| entry.relative.starts_with(&relative))
      {
        return Ok(relative);
      }
    }

    bail!("File is not managed by tildr: {input}");
  }

  if matches!(mode, PickMode::Home) && !home_path.exists() {
    bail!("File does not exist: {}", home_path.display());
  }

  Ok(home_path)
}
