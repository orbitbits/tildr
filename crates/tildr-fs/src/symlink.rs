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
  symlink_target(path).map(|t| t == target).unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    std::env::temp_dir().join(format!("tildr-test-symlink-{nanos}"))
  }

  #[test]
  fn create_and_check_symlink() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let target = dir.join("target.txt");
    let link = dir.join("link.txt");
    fs::write(&target, "hello").unwrap();

    create_symlink(&target, &link).unwrap();
    assert!(is_symlink(&link));
    assert_eq!(symlink_target(&link), Some(target.clone()));
    assert!(is_symlink_to(&link, &target));

    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn is_symlink_returns_false_for_regular_file() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("regular.txt");
    fs::write(&file, "content").unwrap();

    assert!(!is_symlink(&file));

    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn is_symlink_returns_false_for_nonexistent_path() {
    assert!(!is_symlink(Path::new("/tmp/nonexistent_path_xyz")));
  }

  #[test]
  fn symlink_target_returns_none_for_regular_file() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("regular.txt");
    fs::write(&file, "content").unwrap();

    assert_eq!(symlink_target(&file), None);

    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn is_symlink_to_false_when_target_differs() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let target = dir.join("real_target.txt");
    let wrong = dir.join("wrong_target.txt");
    let link = dir.join("link.txt");
    fs::write(&target, "real").unwrap();
    fs::write(&wrong, "wrong").unwrap();

    create_symlink(&target, &link).unwrap();
    assert!(!is_symlink_to(&link, &wrong));

    fs::remove_dir_all(&dir).ok();
  }
}
