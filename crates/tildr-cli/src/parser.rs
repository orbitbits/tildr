use crate::commands::CliCommands;
use clap::Parser;
use tildr_domain::Commands;

#[derive(Parser, Debug)]
#[command(
  name = "tildr",
  about = "Manage your HOME files and directories with symlinks and Git.",
  long_about = "Manage, reproduce, and control everything in your $HOME — declaratively.",
  version,
  after_help = "\
EXAMPLES:
  $ touch ~/config.toml
  $ tildr add config.toml
  $ tildr status
  $ tildr unlink config.toml
  $ tildr status
  $ tildr apply
  $ tildr status
  $ tildr doctor\n"
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: CliCommands,
}

pub fn parse() -> Commands {
  Cli::parse().command.into()
}
