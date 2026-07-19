use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Add a file or all files from a directory to the repository and replace them with symlinks in HOME",
  after_help = "\
EXAMPLES:
  tildr add .config/nvim/
  tildr add .config/nvim/init.vim
  tildr add .config/nvim/init.vim .config/nvim/lua/plugins.lua
  tildr add .config/nvim/init.vim --quiet
  tildr add .bashrc --profile linux\n"
)]
pub struct Command {
  /// Path(s) to file(s) or directory(s) in HOME to be managed
  #[arg(value_name = "PATHS")]
  pub paths: Option<Vec<String>>,

  /// Target profile (defaults to active profile or 'default')
  #[arg(short, long)]
  pub profile: Option<String>,

  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,

  /// Add to repository without creating a symlink (adds to .tildrignore)
  #[arg(long)]
  pub nolink: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Skip confirmation prompts without asking
  #[arg(short, long)]
  pub force: bool,
}
