use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Restore managed files from the repository back to your HOME (remove symlinks)",
  after_help = "\
EXAMPLES:
  tildr restore
  tildr restore .config/nvim/
  tildr restore .config/nvim/init.vim .config/nvim/lua/plugins.lua
  tildr restore .config/nvim/init.vim
  tildr restore .bashrc --profile archlinux\n"
)]
pub struct Command {
  /// Files or directories to restore. Defaults to interactive picker
  pub targets: Vec<String>,

  /// Resolve targets in a specific profile instead of using the active one
  #[arg(short, long)]
  pub profile: Option<String>,

  /// Restore all managed files to HOME
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
