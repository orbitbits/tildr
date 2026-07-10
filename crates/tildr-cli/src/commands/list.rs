use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "List managed files in the repository",
  after_help = "\
EXAMPLES:
  tildr list
  tildr list --tree\n"
)]
pub struct Command {
  /// Show files in a tree view
  #[arg(short, long)]
  pub tree: bool,
  /// Show detailed information (type, size)
  #[arg(short, long)]
  pub long: bool,
}
