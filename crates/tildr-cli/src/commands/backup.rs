use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Create a backup tarball of the repository",
  after_help = "\
EXAMPLES:
  tildr backup
  tildr backup --output ~/my-backup.tar.gz\n"
)]
pub struct Command {
  /// Custom output path for the backup file
  #[arg(long, value_name = "FILE")]
  pub output: Option<String>,
}
