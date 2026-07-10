use std::process::Command;

pub fn detect_gpg_available() -> bool {
  Command::new("gpg")
    .arg("--version")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}
