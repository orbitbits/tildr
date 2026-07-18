use anyhow::{Result, bail};
use std::io::{self, Write};

/// Prompts the user for confirmation unless force is true.
pub fn confirm(force: bool, prompt: &str) -> Result<()> {
  if force {
    return Ok(());
  }

  print!("{prompt}");
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  if matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
    return Ok(());
  }

  bail!("Aborted")
}
