use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Check managed symlinks and show problems by default",
  after_help = "\
EXAMPLES:
  tildr status
  tildr status --all
  tildr status --less
  tildr status --profile work\n"
)]
pub struct Command {
  /// Show all managed files, including correctly linked files
  #[arg(short, long)]
  pub all: bool,

  /// Output status information in JSON format
  #[arg(short, long)]
  pub json: bool,

  /// Display a summary of managed files and their states
  #[arg(short, long)]
  pub counter: bool,

  /// View the output in an interactive pager
  #[arg(short, long)]
  pub less: bool,

  /// Filter by profile name, or no-profile for shared files
  #[arg(long, value_name = "NAME")]
  pub profile: Option<String>,
}
