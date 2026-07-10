use std::{fs, path::Path};
use walkdir::WalkDir;

pub(super) fn check_repo_permissions(path: &Path) -> bool {
  fs::read_dir(path).is_ok()
    && fs::metadata(path)
      .map(|metadata| !metadata.permissions().readonly())
      .unwrap_or(false)
}

pub(super) fn repo_size(path: &Path) -> u64 {
  WalkDir::new(path)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|entry| entry.file_type().is_file())
    .filter_map(|entry| entry.metadata().ok())
    .map(|metadata| metadata.len())
    .sum()
}
