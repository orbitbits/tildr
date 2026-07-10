use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Push and pull repository changes with the remote",
  after_help = "\
EXAMPLE:
  tildr sync
  tildr sync --quiet\n"
)]
pub struct Command {
  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Skip confirmation prompts without asking
  #[arg(short, long)]
  pub force: bool,
}
