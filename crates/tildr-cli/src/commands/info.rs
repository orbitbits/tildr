use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone)]
#[command(
  about = "Display information about Tildr",
  after_help = "\
EXAMPLES:
  tildr info credits
  tildr info license\n"
)]
pub enum CliInfoMode {
  /// Show license information
  License,
  /// Show project credits and contributors
  Credits,
}

#[derive(Args, Debug, Clone)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliInfoMode,
}
