use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::paths::normalize_lexically;

#[cfg(unix)]
pub fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
  std::os::unix::fs::symlink(src, dst)?;
  Ok(())
}

#[cfg(windows)]
pub fn create_symlink(src: &Path, dst: &Path) -> Result<()> {
  if src.is_dir() {
    std::os::windows::fs::symlink_dir(src, dst)?;
  } else {
    std::os::windows::fs::symlink_file(src, dst)?;
  }
  Ok(())
}

pub fn is_symlink(path: &Path) -> bool {
  path
    .symlink_metadata()
    .map(|m| m.file_type().is_symlink())
    .unwrap_or(false)
}

pub fn symlink_target(path: &Path) -> Option<PathBuf> {
  std::fs::read_link(path).ok()
}

pub fn symlink_target_absolute(path: &Path) -> Option<PathBuf> {
  let target = symlink_target(path)?;
  let absolute = if target.is_absolute() {
    target
  } else {
    path
      .parent()
      .map_or(target.clone(), |parent| parent.join(target))
  };

  Some(normalize_lexically(&absolute))
}

pub fn is_symlink_within(path: &Path, root: &Path) -> bool {
  symlink_target_absolute(path).is_some_and(|target| target.starts_with(root))
}

pub fn is_symlink_to(path: &Path, target: &Path) -> bool {
  let Some(absolute_link_target) = symlink_target_absolute(path) else {
    return false;
  };

  paths_match(&absolute_link_target, target)
}

fn paths_match(left: &Path, right: &Path) -> bool {
  if left == right {
    return true;
  }

  match (left.canonicalize(), right.canonicalize()) {
    (Ok(left), Ok(right)) => left == right,
    _ => false,
  }
}
