use std::path::PathBuf;

use anyhow::Result;

pub struct RepoManager {
  pub repo_path: PathBuf,
}

impl RepoManager {
  pub fn new(repo_path: PathBuf) -> Self {
    Self { repo_path }
  }

  pub fn exists(&self) -> bool {
    self.repo_path.exists()
  }

  pub fn init(&self) -> Result<()> {
    std::fs::create_dir_all(&self.repo_path)?;
    Ok(())
  }

  pub fn repo_path_for(&self, relative: &std::path::Path) -> PathBuf {
    self.repo_path.join(relative)
  }
}
