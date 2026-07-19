use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Show the current status of managed files and symlinks",
  after_help = "\
EXAMPLES:
  tildr status
  tildr status --long
  tildr status --less
  tildr status --profile work\n"
)]
pub struct Command {
  /// Output status information in JSON format
  #[arg(short, long)]
  pub json: bool,

  /// Display a summary of managed files and their states
  #[arg(short, long)]
  pub counter: bool,

  /// Show the repository source path in a separate SOURCE column
  #[arg(long)]
  pub long: bool,

  /// View the output in an interactive pager
  #[arg(short, long)]
  pub less: bool,

  /// Filter by profile name
  #[arg(long, value_name = "NAME")]
  pub profile: Option<String>,
}
