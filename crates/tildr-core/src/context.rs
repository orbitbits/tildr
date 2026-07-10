use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Context {
  pub config: Config,
  pub repo_path: PathBuf,
  pub home_path: PathBuf,
}

impl Context {
  pub fn load() -> Result<Self> {
    let config = Config::load()?;
    let repo_path = config.repo_path();
    let home_path =
      dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(Context {
      config,
      repo_path,
      home_path,
    })
  }

  pub fn with_repo(repo: PathBuf) -> Result<Self> {
    let mut config = Config::default();
    config.core.repo = repo.to_string_lossy().to_string();
    let home_path =
      dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(Context {
      repo_path: repo,
      config,
      home_path,
    })
  }
}
