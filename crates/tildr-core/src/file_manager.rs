use crate::config::Config;
use anyhow::{Context as _, Result};
use std::{path::Path, process::Command};

pub fn configured_file_manager(config: &Config) -> Option<&str> {
  let file_manager = config.core.file_manager.trim();
  (!file_manager.is_empty()).then_some(file_manager)
}

pub fn open_directory(config: &Config, path: &Path) -> Result<()> {
  if open_directory_with_configured(config, path)? {
    return Ok(());
  }

  open::that_detached(path).context("Failed to open file manager")?;
  Ok(())
}

pub fn open_directory_with_configured(config: &Config, path: &Path) -> Result<bool> {
  let Some(file_manager) = configured_file_manager(config) else {
    return Ok(false);
  };

  Command::new(file_manager)
    .arg(path)
    .spawn()
    .with_context(|| format!("Failed to open file manager: {file_manager}"))?;

  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::configured_file_manager;
  use crate::Config;

  #[test]
  fn configured_file_manager_returns_none_when_empty() {
    let config = Config::default();

    assert_eq!(configured_file_manager(&config), None);
  }

  #[test]
  fn configured_file_manager_trims_value() {
    let mut config = Config::default();
    config.core.file_manager = "  nautilus  ".to_string();

    assert_eq!(configured_file_manager(&config), Some("nautilus"));
  }
}
