// (c) 2026 OrbitBits. All rights reserved.
//! Entry point for command-line application.

use anyhow::Result;

/// MAIN
fn main() -> Result<()> {
  disable_color_if_needed();

  let command = tildr_cli::parse();
  tildr_commands::dispatch(command)
}

/// Disable all color terminal
fn disable_color_if_needed() {
  if std::env::var("NO_COLOR").is_ok() {
    return;
  }

  if let Ok(config) = tildr_core::config::Config::load()
    && !config.core.color
  {
    unsafe {
      std::env::set_var("NO_COLOR", "1");
    }
  }
}
