use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "List managed files in the repository",
  after_help = "\
EXAMPLES:
  tildr list
  tildr list --tree
  tildr list --export ~/tildr-files.json
  tildr list --import ~/tildr-files.json\n"
)]
pub struct Command {
  /// Show files in a tree view
  #[arg(short, long)]
  pub tree: bool,
  /// Show detailed information (type, size)
  #[arg(short, long)]
  pub long: bool,
  /// Export managed file list to a JSON file
  #[arg(long, value_name = "FILE")]
  pub export: Option<String>,
  /// Import managed file list from a JSON file and create symlinks
  #[arg(long, value_name = "FILE")]
  pub import: Option<String>,
}
