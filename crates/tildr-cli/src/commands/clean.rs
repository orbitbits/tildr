use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Remove empty directories left in profile storage",
  after_help = "\
EXAMPLES:
  tildr clean
  tildr clean --dry-run
  tildr clean --quiet\n"
)]
pub struct Command {
  /// Show what would be removed without deleting directories
  #[arg(short, long)]
  pub dry_run: bool,

  /// Suppress per-directory output
  #[arg(short, long)]
  pub quiet: bool,
}
