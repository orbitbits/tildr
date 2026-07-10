use anyhow::Result;
use std::process::Command;

pub struct DocsArgs {
  pub config: bool,
}

pub fn run(args: DocsArgs) -> Result<()> {
  if args.config {
    let status = Command::new("man").arg("tildr-config").status();
    match status {
      Ok(s) if s.success() => Ok(()),
      _ => fallback(),
    }
  } else {
    Ok(())
  }
}

fn fallback() -> Result<()> {
  eprintln!("man not available or page not installed.");
  eprintln!("See online documentation:");
  eprintln!("https://orbitbits.github.io/products/tildr/documentation/");
  Ok(())
}
