use std::path::{Path, PathBuf};

pub fn expand_home(path: &str) -> PathBuf {
  // HOME
  if path == "~" {
    return dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
  }

  if let Some(p) = path.strip_prefix("~/") {
    return dirs::home_dir()
      .unwrap_or_else(|| PathBuf::from("/"))
      .join(p);
  }

  // It's already absolute → it returns directly
  let p = Path::new(path);
  if p.is_absolute() {
    return p.to_path_buf();
  }

  // Otherwise → path relative to CWD
  std::env::current_dir()
    .unwrap_or_else(|_| PathBuf::from("."))
    .join(p)
}

// UNUSED
// pub fn collapse_home(path: PathBuf) -> String {
//   if let Some(home) = dirs::home_dir()
//     && let Ok(rel) = path.strip_prefix(&home)
//   {
//     return format!("~/{}", rel.display());
//   }
//   path.display().to_string()
// }

pub fn resolve_home_path(input: &str, home: &Path) -> PathBuf {
  let input_path = Path::new(input);

  // It resolves when you use ~/...
  if input.starts_with("~/") {
    return home.join(input.trim_start_matches("~/"));
  }

  // Absolute path
  if input_path.is_absolute() {
    return input_path.to_path_buf();
  }

  // Everything else → related to HOME
  home.join(input_path)
}

#[cfg(test)]
mod tests {
  use super::*;

  // --- resolve_home_path ---

  #[test]
  fn resolve_home_path_tilde_prefix() {
    let home = PathBuf::from("/home/user");
    assert_eq!(resolve_home_path("~/.bashrc", &home), home.join(".bashrc"));
  }

  #[test]
  fn resolve_home_path_absolute_path() {
    let home = PathBuf::from("/home/user");
    assert_eq!(
      resolve_home_path("/etc/config", &home),
      PathBuf::from("/etc/config")
    );
  }

  #[test]
  fn resolve_home_path_relative_path() {
    let home = PathBuf::from("/home/user");
    assert_eq!(
      resolve_home_path("docs/file.txt", &home),
      home.join("docs/file.txt")
    );
  }

  #[test]
  fn resolve_home_path_dot_prefix() {
    let home = PathBuf::from("/home/user");
    assert_eq!(resolve_home_path("./local", &home), home.join("./local"));
  }

  // --- expand_home (absolute only — HOME-dependent cases need integration) ---

  #[test]
  fn expand_home_absolute_path_unchanged() {
    let result = expand_home("/etc/config");
    assert_eq!(result, PathBuf::from("/etc/config"));
  }

  #[test]
  fn expand_home_relative_path_joins_cwd() {
    let cwd = std::env::current_dir().unwrap();
    let result = expand_home("relative/path");
    assert_eq!(result, cwd.join("relative/path"));
  }
}
