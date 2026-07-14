use anyhow::Result;
use std::{
  collections::hash_map::DefaultHasher,
  fs,
  hash::{Hash, Hasher},
  io,
  path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

// IMPORTANT: These files are not considered user files, so they will be
// ignored by default.
//
// Internal Tildr directory name
const TILDR_DIR: &str = ".tildr";

// List of ignored files in all repository Tildr
const IGNORE_FILES: &[&str] = &[
  ".DS_Store",
  "Thumbs.db",
  ".gitkeep",
  ".gitignore",
  ".tildrignore",
];
//
// List of ignored extensions in all repository Tildr
const IGNORE_EXTENSION: &[&str] = &["bak", "tmp", "swp"];
//
// List of ignored suffixes in all repository Tildr
const IGNORE_SUFFIXES: &[&str] = &["~"];

/// Returns the path to the `.tildr/` internal directory.
/// Creates the directory if it does not exist.
pub fn tildr_dir(repo_path: &Path) -> PathBuf {
  let dir = repo_path.join(TILDR_DIR);
  let _ = std::fs::create_dir_all(&dir);
  dir
}

pub fn should_ignore(path: &Path) -> bool {
  if let Some(name) = path.file_name().and_then(|n| n.to_str())
    && name == TILDR_DIR
  {
    return true;
  }

  let file_name = match path.file_name().and_then(|n| n.to_str()) {
    Some(n) => n,
    None => return false,
  };

  // Nome exato
  if IGNORE_FILES.contains(&file_name) {
    return true;
  }

  // Sufixo tipo *~
  if IGNORE_SUFFIXES.iter().any(|s| file_name.ends_with(s)) {
    return true;
  }

  // Extensão
  if let Some(ext) = path.extension().and_then(|e| e.to_str())
    && IGNORE_EXTENSION.contains(&ext.to_ascii_lowercase().as_str())
  {
    return true;
  }

  false
}

pub fn format_size(size: u64) -> String {
  const KB: u64 = 1024;
  const MB: u64 = KB * 1024;

  if size >= MB {
    format!("{:.1}M", size as f64 / MB as f64)
  } else if size >= KB {
    format!("{:.1}K", size as f64 / KB as f64)
  } else {
    format!("{}B", size)
  }
}

pub fn short_id(path: &str) -> String {
  let mut hasher = DefaultHasher::new();
  path.hash(&mut hasher);
  format!("{:x}", hasher.finish())[0..4].to_string()
}

pub fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
  match fs::rename(src, dst) {
    Ok(_) => Ok(()),
    Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
      fs::copy(src, dst)?;
      fs::remove_file(src)?;
      Ok(())
    }
    Err(e) => Err(e),
  }
}

pub fn move_dir(src: &Path, dst: &Path) -> io::Result<()> {
  copy_dir_all(src, dst)?;
  fs::remove_dir_all(src)?;
  Ok(())
}

pub fn move_to_trash(path: &Path) -> Result<()> {
  #[cfg(target_os = "linux")]
  if gio_trash(path).is_ok() {
    return Ok(());
  }

  trash::delete(path)?;
  Ok(())
}

#[cfg(target_os = "linux")]
fn gio_trash(path: &Path) -> Result<()> {
  let mut cmd = Command::new("gio");
  cmd.arg("trash").arg(path);

  if !cfg!(debug_assertions) {
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
  }

  let status = cmd.status()?;

  if !status.success() {
    anyhow::bail!("Failed to move to trash using gio");
  }

  Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
  fs::create_dir_all(dst)?;

  for entry in fs::read_dir(src)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let dest_path = dst.join(entry.file_name());

    if file_type.is_dir() {
      copy_dir_all(&entry.path(), &dest_path)?;
    } else {
      fs::copy(entry.path(), &dest_path)?;
    }
  }

  Ok(())
}

pub fn ensure_parent(path: &Path) -> io::Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  Ok(())
}

pub fn remove_path(path: &Path) -> io::Result<()> {
  if path.is_dir() {
    fs::remove_dir_all(path)
  } else {
    fs::remove_file(path)
  }
}
