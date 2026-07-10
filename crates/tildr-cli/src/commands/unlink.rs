use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Remove symlinks from your HOME without touching repository files",
  after_help = "\
EXAMPLES:
  tildr unlink
  tildr unlink .config/nvim/
  tildr unlink .config/nvim/init.vim .config/nvim/lua/plugins.lua
  tildr unlink .config/nvim/init.vim\n"
)]
pub struct Command {
  /// File(s) or directory(s) to unlink (auto-detected). If not provided, an interactive picker will be shown
  pub targets: Vec<String>,

  /// Unlink all managed files from HOME
  #[arg(short, long, conflicts_with = "targets")]
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
}
