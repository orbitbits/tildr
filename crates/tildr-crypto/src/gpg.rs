use anyhow::{Result, bail};
use std::{
  path::{Path, PathBuf},
  process::Command,
};

pub struct GpgIntegration {
  repo_path: PathBuf,
}

impl GpgIntegration {
  pub fn new(repo_path: &Path) -> Self {
    Self {
      repo_path: repo_path.to_path_buf(),
    }
  }

  pub fn bundle_path(&self) -> PathBuf {
    self.repo_path.join(".tildr-encrypt.gpg")
  }

  /// List available secret keys as (key_id, uid) pairs
  pub fn list_secret_keys(&self) -> Result<Vec<(String, String)>> {
    let output = Command::new("gpg")
      .args([
        "--list-secret-keys",
        "--keyid-format",
        "LONG",
        "--with-colons",
      ])
      .output()?;

    if !output.status.success() {
      bail!(
        "gpg --list-secret-keys failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut keys: Vec<(String, String)> = Vec::new();
    let mut current_key_id = String::new();
    let mut uid_collected = false;

    for line in stdout.lines() {
      let parts: Vec<&str> = line.split(':').collect();
      let record_type = parts.first().copied().unwrap_or("");

      // Match "sec" and "sec#" (offline primary key) but NOT "ssb" (subkeys)
      if record_type == "sec" || record_type == "sec#" {
        current_key_id = parts.get(4).unwrap_or(&"").to_string();
        uid_collected = false;
        continue;
      }

      if record_type == "uid" && !current_key_id.is_empty() && !uid_collected {
        let uid = parts.get(9).unwrap_or(&"").to_string();
        if !uid.is_empty() {
          keys.push((current_key_id.clone(), uid));
          uid_collected = true;
        }
      }
    }

    // Deduplicate by key_id — same key appearing twice (e.g. in multiple keyrings)
    let mut seen = std::collections::HashSet::new();
    keys.retain(|(key_id, _)| seen.insert(key_id.clone()));

    Ok(keys)
  }

  /// Encrypt using symmetric mode (passphrase only)
  pub fn encrypt_symmetric(&self, entries: &[String], home_path: &Path) -> Result<()> {
    self.validate_entries(entries, home_path)?;

    let tar_path = self.repo_path.join(".tildr-encrypt.tar");
    self.create_tar(entries, home_path, &tar_path)?;

    let bundle = self.bundle_path();
    let status = Command::new("gpg")
      .args([
        "--batch",
        "--yes",
        "--symmetric",
        "--cipher-algo",
        "AES256",
        "--output",
        &bundle.to_string_lossy(),
        &tar_path.to_string_lossy(),
      ])
      .status()?;

    let _ = std::fs::remove_file(&tar_path);

    if !status.success() {
      bail!("gpg symmetric encryption failed");
    }

    Ok(())
  }

  /// Encrypt using asymmetric mode (public key of recipient)
  pub fn encrypt_asymmetric(
    &self,
    entries: &[String],
    home_path: &Path,
    recipient: &str,
  ) -> Result<()> {
    if recipient.is_empty() {
      bail!("No GPG key configured for asymmetric encryption. Set [crypto].gpg_key in config.toml");
    }

    self.validate_entries(entries, home_path)?;

    let tar_path = self.repo_path.join(".tildr-encrypt.tar");
    self.create_tar(entries, home_path, &tar_path)?;

    let bundle = self.bundle_path();
    let status = Command::new("gpg")
      .args([
        "--batch",
        "--yes",
        "--recipient",
        recipient,
        "--encrypt",
        "--output",
        &bundle.to_string_lossy(),
        &tar_path.to_string_lossy(),
      ])
      .status()?;

    let _ = std::fs::remove_file(&tar_path);

    if !status.success() {
      bail!(
        "gpg asymmetric encryption failed for recipient: {}",
        recipient
      );
    }

    Ok(())
  }

  /// Decrypt the bundle (works for both symmetric and asymmetric)
  pub fn decrypt(&self, home_path: &Path) -> Result<()> {
    let bundle = self.bundle_path();
    if !bundle.exists() {
      bail!("No encrypted bundle found: {}", bundle.display());
    }

    let tar_path = self.repo_path.join(".tildr-decrypt.tar");

    let status = Command::new("gpg")
      .args([
        "--batch",
        "--yes",
        "--decrypt",
        "--output",
        &tar_path.to_string_lossy(),
        &bundle.to_string_lossy(),
      ])
      .status()?;

    if !status.success() {
      let _ = std::fs::remove_file(&tar_path);
      bail!("gpg decryption failed");
    }

    let tar_status = Command::new("tar")
      .args([
        "xf",
        &tar_path.to_string_lossy(),
        "-C",
        &home_path.to_string_lossy(),
      ])
      .status()?;

    let _ = std::fs::remove_file(&tar_path);

    if !tar_status.success() {
      bail!("tar extraction failed");
    }

    Ok(())
  }

  fn validate_entries(&self, entries: &[String], home_path: &Path) -> Result<()> {
    if entries.is_empty() {
      bail!("No files listed in .tildr-encrypt");
    }
    for entry in entries {
      let full = home_path.join(entry);
      if !full.exists() {
        bail!("File not found: {}", full.display());
      }
    }
    Ok(())
  }

  fn create_tar(&self, entries: &[String], home_path: &Path, tar_path: &Path) -> Result<()> {
    let mut args = vec![
      "cf".to_string(),
      tar_path.to_string_lossy().to_string(),
      "-C".to_string(),
      home_path.to_string_lossy().to_string(),
    ];
    for entry in entries {
      args.push(entry.clone());
    }

    let status = Command::new("tar").args(&args).status()?;
    if !status.success() {
      bail!("tar failed while creating archive");
    }
    Ok(())
  }
}
