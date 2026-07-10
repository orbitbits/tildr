use crate::parser::Cli;
use anyhow::{Result, anyhow};
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;
use std::str::FromStr;

pub fn generate_completions(shell: &str) -> Result<()> {
  let shell = Shell::from_str(shell).map_err(|_| anyhow!("Unsupported shell: {}", shell))?;

  let mut cmd = Cli::command();
  generate(shell, &mut cmd, "tildr", &mut io::stdout());

  Ok(())
}
