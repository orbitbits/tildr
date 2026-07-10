use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Check the health of the Tildr environment and detect issues",
  after_help = "\
EXAMPLE:
  tildr doctor\n"
)]
pub struct Command;
