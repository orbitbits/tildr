use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Show the current status of managed files and symlinks",
  after_help = "\
EXAMPLE:
  tildr status\n"
)]
pub struct Command {
  /// Output status information in JSON format
  #[arg(short, long)]
  pub json: bool,

  /// Display a summary of managed files and their states
  #[arg(short, long)]
  pub counter: bool,
}
