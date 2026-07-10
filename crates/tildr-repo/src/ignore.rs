use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

pub struct IgnoreMatcher {
  matcher: Gitignore,
  root: PathBuf,
}

impl IgnoreMatcher {
  /// Creates a matcher based on `.tildrignore` within the repository.
  pub fn from_repo(repo_path: &Path) -> Result<Self> {
    let mut builder = GitignoreBuilder::new(repo_path);

    let ignore_file = repo_path.join(".tildrignore");

    if ignore_file.exists() {
      builder.add(ignore_file);
    }

    let matcher = builder.build()?;

    Ok(Self {
      matcher,
      root: repo_path.to_path_buf(),
    })
  }

  /// Checks whether a path should be ignored.
  ///
  /// IMPORTANT:
  /// - Wait for the path relative to the repository (or convert it internally).
  pub fn is_ignored(&self, path: &Path) -> bool {
    let relative = match path.strip_prefix(&self.root) {
      Ok(p) => p,
      Err(_) => path,
    };

    self
      .matcher
      .matched_path_or_any_parents(relative, path.is_dir())
      .is_ignore()
  }
}
