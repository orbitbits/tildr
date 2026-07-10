use anyhow::Result;
use std::{fs, path::Path};
use tildr_fs::symlink::is_symlink;
use tildr_ui::warn;
use tildr_utils::fs::move_to_trash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionMode {
  Trash,
  Purge,
}

impl DeletionMode {
  pub fn label(self) -> &'static str {
    match self {
      Self::Trash => "trash",
      Self::Purge => "purge",
    }
  }

  fn remove(self, path: &Path) -> Result<()> {
    match self {
      Self::Trash => move_to_trash(path),
      Self::Purge if path.is_dir() => {
        fs::remove_dir_all(path)?;
        Ok(())
      }
      Self::Purge => {
        fs::remove_file(path)?;
        Ok(())
      }
    }
  }
}

impl From<bool> for DeletionMode {
  fn from(purge: bool) -> Self {
    if purge { Self::Purge } else { Self::Trash }
  }
}

pub struct ManagedPathOp<'a> {
  home: &'a Path,
  repo: &'a Path,
  relative: &'a Path,
}

impl<'a> ManagedPathOp<'a> {
  pub fn new(home: &'a Path, repo: &'a Path, relative: &'a Path) -> Self {
    Self {
      home,
      repo,
      relative,
    }
  }

  pub fn restore(&self) -> Result<()> {
    self.remove_home_symlink_if_present()?;

    if let Some(parent) = self.home.parent() {
      fs::create_dir_all(parent)?;
    }

    fs::rename(self.repo, self.home)?;
    Ok(())
  }

  pub fn delete(&self, mode: DeletionMode) -> Result<()> {
    self.remove_home_symlink_if_present()?;

    if self.repo.exists() {
      mode.remove(self.repo)?;
    }

    Ok(())
  }

  pub fn unlink(&self) -> Result<bool> {
    if self.home.symlink_metadata().is_err() || !is_symlink(self.home) {
      return Ok(false);
    }

    fs::remove_file(self.home)?;
    Ok(true)
  }

  fn remove_home_symlink_if_present(&self) -> Result<()> {
    if self.home.symlink_metadata().is_ok() {
      if is_symlink(self.home) {
        fs::remove_file(self.home)?;
      } else {
        warn(&format!("{} is not a symlink", self.relative.display()));
      }
    }

    Ok(())
  }
}

pub fn cleanup_empty_ancestors(root: &Path, relative: &Path) {
  let mut current = root.join(relative).parent().map(|path| path.to_path_buf());

  while let Some(dir) = current {
    if dir == root {
      break;
    }

    if fs::read_dir(&dir)
      .map(|mut entries| entries.next().is_none())
      .unwrap_or(false)
    {
      let _ = fs::remove_dir(&dir);
      current = dir.parent().map(|path| path.to_path_buf());
    } else {
      break;
    }
  }
}
