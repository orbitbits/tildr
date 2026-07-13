use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Suggest files in $HOME that could be managed by Tildr",
  after_help = "\
EXAMPLES:
  tildr suggest\n"
)]
pub struct Command {}
