use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Create and update symlinks in HOME from the repository",
  after_help = "\
EXAMPLE:
  tildr apply\n"
)]
pub struct Command {
  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,

  /// Show detailed output of operations performed
  #[arg(short, long)]
  pub verbose: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Skip confirmation prompts without asking
  #[arg(short, long)]
  pub force: bool,
}
