use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Delete files from the repository and unlink from HOME",
  after_help = "\
EXAMPLES:
  tildr del
  tildr del .config/nvim/
  tildr del .config/nvim/init.vim\n"
)]
pub struct Command {
  /// Files or directories to restore. Defaults to interactive picker
  pub target: Option<String>,

  /// Delete all managed files in the repo
  #[arg(short, long, conflicts_with = "target")]
  pub all: bool,

  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Skip confirmation prompts without asking
  #[arg(short, long)]
  pub force: bool,

  /// This will permanently delete the files from the repository without sending them to the trash and will unlink them from the HOME directory. (DANGEROUS)
  #[arg(short, long)]
  pub purge: bool,
}
