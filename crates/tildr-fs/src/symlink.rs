use anyhow::Result;
use std::path::{Path, PathBuf};

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

pub fn is_symlink_to(path: &Path, target: &Path) -> bool {
  let Some(link_target) = symlink_target(path) else {
    return false;
  };

  let absolute_link_target = if link_target.is_absolute() {
    link_target
  } else {
    path
      .parent()
      .map(|parent| parent.join(&link_target))
      .unwrap_or(link_target)
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
