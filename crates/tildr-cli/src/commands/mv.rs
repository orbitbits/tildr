use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Rename or move a managed file and update its symlink",
  after_help = "\
EXAMPLES:
  tildr mv .bashrc .bashrc_backup
  tildr mv files/file.txt configs/file.txt
  tildr mv\n"
)]
pub struct Command {
  /// Current relative path of the managed file
  pub source: Option<String>,

  /// New relative path or filename
  pub dest: Option<String>,

  /// Show what would be done without making changes
  #[arg(short, long)]
  pub dry_run: bool,

  /// Suppress output
  #[arg(short, long)]
  pub quiet: bool,
}
