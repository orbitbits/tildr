use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone)]
#[command(
  about = "Manage and inspect the Tildr repository",
  after_help = "\
EXAMPLE:
  tildr repo path
  cd $(tildr repo path)\n"
)]
pub enum CliRepoMode {
  /// Print the path to the Tildr repository
  Path,
}

#[derive(Args, Debug, Clone)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliRepoMode,
}
