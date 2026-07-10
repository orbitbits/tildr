use thiserror::Error;

#[derive(Debug, Error)]
pub enum TildrError {
  #[error("Repository not found at {0}")]
  RepoNotFound(String),

  #[error("Config not found")]
  ConfigNotFound,

  #[error("File not found: {0}")]
  FileNotFound(String),

  #[error("Path must be inside HOME directory")]
  PathOutsideHome,

  #[error("File is already")]
  AlreadyManaged,

  #[error("File is not")]
  NotManaged,

  #[error("Symlink target is broken: {0}")]
  BrokenSymlink(String),
}
