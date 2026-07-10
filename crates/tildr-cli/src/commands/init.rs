use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Initialize a Tildr repository and configuration",
  after_help = "\
EXAMPLES:
  tildr init
  tildr init --repo ~/my_files\n"
)]
pub struct Command {
  /// Specify the repository directory (must be inside HOME)
  #[arg(short, long)]
  pub repo: Option<String>,

  /// Do not initialize a Git repository
  #[arg(long)]
  pub no_git: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,

  /// Skip confirmation prompts without asking
  #[arg(short, long)]
  pub force: bool,
}
