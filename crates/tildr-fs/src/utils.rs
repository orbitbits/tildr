use anyhow::Result;
use std::{
  fs,
  path::{Path, PathBuf},
};

pub fn backup_path(path: &Path) -> PathBuf {
  let name = path
    .file_name()
    .map(|n| format!("{}.bak", n.to_string_lossy()))
    .unwrap_or_else(|| "backup.bak".to_string());
  path.with_file_name(name)
}

/*
Backups can cause a lot of problems with the accumulation of .bak files.
For better user adaptation, we will investigate further to add this feature in future versions.
*/

// pub fn backup_file(path: &Path) -> Result<PathBuf> {
//   let backup = backup_path(path);
//   if path.is_file() {
//     fs::copy(path, &backup)?;
//   }
//   Ok(backup)
// }

pub fn remove_file_or_dir(path: &Path) -> Result<()> {
  let meta = path.symlink_metadata()?;
  if meta.file_type().is_symlink() || meta.file_type().is_file() {
    fs::remove_file(path)?;
  } else if meta.file_type().is_dir() {
    fs::remove_dir_all(path)?;
  }
  Ok(())
}
