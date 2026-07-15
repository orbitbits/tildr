use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Generate a reproducible bootstrap script from the current setup",
  long_about = "Generate a shell script that reproduces your Tildr setup on a new machine.\n\nThe script handles cloning the repository, initializing Tildr, applying symlinks, and restoring secrets.\n\nOutput is written to stdout by default — redirect to a file to save it.",
  after_help = "\
EXAMPLES:
  tildr snapshot > setup.sh
  tildr snapshot --output ~/setup.sh
  chmod +x setup.sh
  ./setup.sh\n"
)]
pub struct Command {
  /// Custom output file path. If omitted, prints to stdout.
  #[arg(long, value_name = "FILE")]
  pub output: Option<String>,
}
