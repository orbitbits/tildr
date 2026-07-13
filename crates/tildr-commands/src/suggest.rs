use std::collections::HashMap;

use anyhow::{Context as _, Result};
use console::style;
use tildr_core::context::Context;

pub fn run(ctx: &Context) -> Result<()> {
  let home = dirs::home_dir().context("Could not determine home directory")?;
  let entries = tildr_repo::scatildr_repo(&ctx.repo_path)?;

  let managed: std::collections::HashSet<_> = entries.iter().map(|e| e.relative.clone()).collect();

  let mut suggestions: HashMap<&str, Vec<String>> = HashMap::new();

  let common_files: &[(&str, &[&str])] = &[
    (
      "Shell",
      &[".bashrc", ".zshrc", ".bash_profile", ".profile", ".zshenv"],
    ),
    ("Editor", &[".vimrc", ".editorconfig"]),
    ("Git", &[".gitconfig", ".gitignore_global"]),
    ("Terminal", &[".tmux.conf", ".inputrc"]),
    ("Tools", &[".fzf.zsh", ".ripgreprc", ".fdignore"]),
  ];

  let common_dirs: &[(&str, &[&str])] = &[
    ("Editor", &[".config/nvim", ".vscode"]),
    (
      "Terminal",
      &[
        ".config/alacritty",
        ".config/kitty",
        ".config/wezterm",
        ".config/foot",
      ],
    ),
    ("Shell", &[".config/fish", ".config/zsh"]),
    (
      "Tools",
      &[
        ".config/rofi",
        ".config/dunst",
        ".config/hypr",
        ".config/sway",
        ".config/i3",
      ],
    ),
    (
      "DE",
      &[
        ".config/gtk-3.0",
        ".config/qt5ct",
        ".config/electron-flags.conf",
      ],
    ),
  ];

  for (category, files) in common_files {
    for file in *files {
      let path = home.join(file);
      if path.exists() && !managed.contains(std::path::Path::new(file)) {
        suggestions
          .entry(*category)
          .or_default()
          .push(file.to_string());
      }
    }
  }

  for (category, dirs_list) in common_dirs {
    for dir in *dirs_list {
      let path = home.join(dir);
      if path.exists() && !managed.contains(std::path::Path::new(dir)) {
        suggestions
          .entry(*category)
          .or_default()
          .push(dir.to_string());
      }
    }
  }

  if suggestions.is_empty() {
    println!(
      "{}",
      style("No suggestions. Everything looks managed!").green()
    );
    return Ok(());
  }

  println!("{}", style("Suggested files in $HOME:").bold());
  println!();

  let mut sorted: Vec<_> = suggestions.into_iter().collect();
  sorted.sort_by_key(|(cat, _)| cat.to_string());

  for (category, files) in &sorted {
    let display: Vec<_> = files.iter().map(|f| style(f).cyan().to_string()).collect();
    println!(
      "  {:<14} {}",
      style(format!("{}:", category)).bold(),
      display.join(", ")
    );
  }

  println!();
  println!("{}", style("Run `tildr add <file>` to manage them.").dim());

  Ok(())
}
