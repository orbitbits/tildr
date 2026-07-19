use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "List managed files as HOME paths",
  after_help = "\
EXAMPLES:
  tildr list
  tildr list --tree
  tildr list --less
  tildr list --export ~/tildr-files.json
  tildr list --import ~/tildr-files.json
  tildr list --profile work
  tildr list --long --profile common\n"
)]
pub struct Command {
  /// Show files in a tree view
  #[arg(short, long)]
  pub tree: bool,
  /// Show detailed HOME path information (type, size)
  #[arg(short, long)]
  pub long: bool,
  /// Show repository source paths instead of HOME paths
  #[arg(long, conflicts_with_all = ["tree", "long"])]
  pub source: bool,
  /// Export managed file list to a JSON file
  #[arg(long, value_name = "FILE")]
  pub export: Option<String>,
  /// Import managed file list from a JSON file and create symlinks
  #[arg(long, value_name = "FILE")]
  pub import: Option<String>,
  /// View the output in an interactive pager
  #[arg(long)]
  pub less: bool,
  /// Filter by profile name
  #[arg(long, value_name = "NAME")]
  pub profile: Option<String>,
}
