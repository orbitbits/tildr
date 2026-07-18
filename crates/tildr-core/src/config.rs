use super::constants::APP_NAME;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
  pub core: Core,
  // pub link: Link,
  pub git: Git,
  pub crypto: Crypto,
}

/*
Backups can cause a lot of problems with the accumulation of .bak files.
For better user adaptation, we will investigate further to add this feature in future versions.
*/
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Core {
  pub repo: String,
  pub search_threshold: usize,
  pub color: bool,
  // pub backup: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Link {
  pub strategy: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Crypto {
  pub mode: CryptoMode,
  /// GPG key ID or email — used only when mode = "asymmetric"
  /// Empty string means: prompt user to select interactively on first use
  #[serde(skip_serializing_if = "String::is_empty")]
  pub gpg_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CryptoMode {
  Symmetric,
  Asymmetric,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Git {
  // pub remote_url: String,
  pub available: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enable: Option<bool>,
  pub auto_commit: bool,
}

impl Default for Crypto {
  fn default() -> Self {
    Self {
      mode: CryptoMode::Symmetric,
      gpg_key: String::new(),
    }
  }
}

impl Default for Core {
  fn default() -> Self {
    Self {
      repo: "~/.dotfiles".to_string(),
      search_threshold: 15,
      color: true,
    }
  }
}

impl Default for Git {
  fn default() -> Self {
    Self {
      available: true,
      enable: None,
      auto_commit: true,
    }
  }
}

impl Config {
  pub fn config_path() -> PathBuf {
    dirs::config_dir()
      .unwrap_or_else(|| {
        dirs::home_dir()
          .map(|h| h.join(".config"))
          .unwrap_or_else(|| PathBuf::from("."))
      })
      .join(APP_NAME)
      .join("config.toml")
  }

  pub fn load() -> Result<Self> {
    let path = Self::config_path();
    if !path.exists() {
      return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)
      .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: Config = toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;
    Ok(config)
  }

  pub fn save(&self) -> Result<()> {
    let path = Self::config_path();
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(self)?;
    fs::write(&path, content)?;
    Ok(())
  }

  pub fn repo_path(&self) -> PathBuf {
    if self.core.repo.starts_with("~/")
      && let Some(home) = dirs::home_dir()
    {
      return home.join(&self.core.repo[2..]);
    }
    PathBuf::from(&self.core.repo)
  }
}

impl std::fmt::Display for CryptoMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CryptoMode::Symmetric => write!(f, "symmetric"),
      CryptoMode::Asymmetric => write!(f, "asymmetric"),
    }
  }
}

impl Git {
  pub fn operations_enabled(&self) -> bool {
    self.available && self.enable != Some(false)
  }

  pub fn auto_commit_enabled(&self) -> bool {
    self.auto_commit && self.operations_enabled()
  }
}
