use std::path::{Path, PathBuf};

pub fn expand_home(path: &str) -> PathBuf {
  let home = dirs::home_dir();
  let expanded = match home.as_deref() {
    Some(home) if path == "~" || path == "$HOME" => home.to_path_buf(),
    Some(home) => path
      .strip_prefix("~/")
      .or_else(|| path.strip_prefix("$HOME/"))
      .map_or_else(|| PathBuf::from(path), |relative| home.join(relative)),
    None => PathBuf::from(path),
  };

  if expanded.is_absolute() {
    normalize_lexically(&expanded)
  } else {
    normalize_lexically(
      &std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(expanded),
    )
  }
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

  if input == "$HOME" {
    return home.to_path_buf();
  }

  if let Some(path) = input.strip_prefix("$HOME/") {
    return normalize_lexically(&home.join(path));
  }

  if input == "~" {
    return home.to_path_buf();
  }

  // It resolves when you use ~/...
  if input.starts_with("~/") {
    return normalize_lexically(&home.join(input.trim_start_matches("~/")));
  }

  // Absolute path
  if input_path.is_absolute() {
    return normalize_lexically(input_path);
  }

  if let Ok(cwd) = std::env::current_dir()
    && cwd.starts_with(home)
  {
    let cwd_path = cwd.join(input_path);
    if input.starts_with("./") || input.starts_with("../") || cwd_path.exists() {
      return normalize_lexically(&cwd_path);
    }
  }

  normalize_lexically(&home.join(input_path))
}

/// Normalize `.` and `..` components without following symlinks.
pub fn normalize_lexically(path: &Path) -> PathBuf {
  let mut normalized = PathBuf::new();

  for component in path.components() {
    match component {
      std::path::Component::Prefix(_) | std::path::Component::RootDir => {
        normalized.push(component.as_os_str());
      }
      std::path::Component::CurDir => {}
      std::path::Component::ParentDir => {
        if normalized.file_name().is_some() {
          normalized.pop();
        } else if !normalized.has_root() {
          normalized.push(component.as_os_str());
        }
      }
      std::path::Component::Normal(part) => normalized.push(part),
    }
  }

  normalized
}
