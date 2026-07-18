use anyhow::Result;
use std::{
  fs,
  path::{Path, PathBuf},
};

use tildr_utils::fs::tildr_dir;

pub struct EncryptManifest {
  pub(crate) path: PathBuf,
}

impl EncryptManifest {
  pub fn new(repo_path: &Path) -> Self {
    Self {
      path: tildr_dir(repo_path).join("encrypted-items"),
    }
  }

  pub fn exists(&self) -> bool {
    self.path.exists()
  }

  pub fn entries(&self) -> Result<Vec<String>> {
    if !self.path.exists() {
      return Ok(vec![]);
    }
    let content = fs::read_to_string(&self.path)?;
    Ok(
      content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect(),
    )
  }

  pub fn add(&self, entry: &str) -> Result<()> {
    let mut entries = self.entries()?;
    let entry = entry.trim().to_string();
    if !entries.contains(&entry) {
      entries.push(entry);
      self.write(&entries)?;
    }
    Ok(())
  }

  pub fn remove(&self, entry: &str) -> Result<bool> {
    let mut entries = self.entries()?;
    let before = entries.len();
    entries.retain(|e| e != entry.trim());
    if entries.len() < before {
      self.write(&entries)?;
      return Ok(true);
    }
    Ok(false)
  }

  fn write(&self, entries: &[String]) -> Result<()> {
    fs::write(&self.path, entries.join("\n") + "\n")?;
    Ok(())
  }
}
