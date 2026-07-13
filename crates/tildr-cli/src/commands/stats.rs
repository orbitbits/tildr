use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Show statistics about managed files",
  after_help = "\
EXAMPLES:
  tildr stats\n"
)]
pub struct Command {}
