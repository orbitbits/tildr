use anyhow::Result;
use std::{
  env,
  io::{self, Write},
  process::{Command, Stdio},
};

/// Entry point
pub fn page_string(content: &str) -> Result<()> {
  // --- Detect if stdout is a TTY ---
  if !is_tty() {
    print!("{}", content);
    return Ok(());
  }

  // --- Resolve pager ---
  if let Some(pager) = resolve_pager()
    && run_pager(&pager, content).is_ok()
  {
    return Ok(());
  }

  // --- Fallback (silent like git) ---
  print!("{}", content);

  Ok(())
}

/// Resolve pager command (like git)
fn resolve_pager() -> Option<String> {
  // $PAGER
  if let Ok(pager) = env::var("PAGER")
    && !pager.trim().is_empty()
  {
    return Some(pager);
  }

  // default: less with flags
  Some("less -RFX".to_string())
}

/// Run pager with piped stdin
fn run_pager(cmd: &str, content: &str) -> Result<()> {
  let mut parts = cmd.split_whitespace();

  let program = parts.next().unwrap();
  let args: Vec<&str> = parts.collect();

  let mut child = Command::new(program)
    .args(&args)
    .stdin(Stdio::piped())
    .spawn()?;

  if let Some(stdin) = child.stdin.as_mut() {
    stdin.write_all(content.as_bytes())?;
  }

  let status = child.wait()?;

  if status.success() {
    Ok(())
  } else {
    anyhow::bail!("pager failed")
  }
}

/// Check if stdout is a terminal
fn is_tty() -> bool {
  use std::io::IsTerminal;
  io::stdout().is_terminal()
}
