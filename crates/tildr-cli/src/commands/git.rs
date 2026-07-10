use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone)]
#[command(
  about = "Run Git commands scoped to the Tildr repository",
  after_help = "\
EXAMPLE:
  tildr git status\n"
)]
pub enum CliGitMode {
  /// Show the working tree status
  Status,
}

#[derive(Args, Debug, Clone)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliGitMode,
}
