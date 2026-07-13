use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Open the Tildr repository in the system file manager",
  after_help = "\
EXAMPLES:
  tildr open\n"
)]
pub struct Command {}
