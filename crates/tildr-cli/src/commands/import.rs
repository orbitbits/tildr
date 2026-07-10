use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Clone a remote dotfiles repository and apply it",
  after_help = "\
EXAMPLES:
  tildr import https://github.com/user/dotfiles
  tildr import https://github.com/user/dotfiles ~/.dotfiles
  tildr import https://github.com/user/dotfiles --force\n"
)]
pub struct Command {
  /// Git repository URL to clone
  pub url: String,

  /// Local destination path (default: ~/.dotfiles)
  pub dest: Option<String>,

  /// Overwrite existing config.toml if it points to a different repo
  #[arg(short, long)]
  pub force: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,
}
