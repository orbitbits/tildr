use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Create and update symlinks in HOME from the repository",
  after_help = "\
EXAMPLE:
  tildr apply
  tildr apply --check\n"
)]
pub struct Command {
  /// Check whether all managed files are correctly linked without making changes
  #[arg(long, conflicts_with = "dry_run")]
  pub check: bool,

  /// Show what would be done without making changes
  #[arg(short = 'n', long)]
  pub dry_run: bool,

  /// Show detailed output of operations performed
  #[arg(short, long)]
  pub verbose: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Replace conflicting regular files or directories in HOME
  #[arg(short, long)]
  pub force: bool,
}
