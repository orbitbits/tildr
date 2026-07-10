use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage .tildrignore patterns",
  after_help = "\
EXAMPLES:
  tildr exclude add *.log
  tildr exclude add cache/
  tildr exclude remove *.log
  tildr exclude list\n"
)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliExcludeMode,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliExcludeMode {
  /// Add a pattern to .tildrignore
  Add {
    /// Gitignore-style pattern (e.g. *.log, cache/, .env)
    pattern: String,
  },

  /// Remove a pattern from .tildrignore
  Remove {
    /// The exact pattern to remove
    pattern: String,
  },

  /// List all patterns in .tildrignore
  List,
}
