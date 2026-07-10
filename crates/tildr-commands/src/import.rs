use anyhow::{Result, bail};
use std::process::Command;
use tildr_core::config::Config;
use tildr_crypto::{EncryptManifest, GpgIntegration, detect_gpg_available};
use tildr_fs::paths::expand_home;
use tildr_ui::{info, success, warn};

pub struct ImportArgs {
  pub url: String,
  pub dest: Option<String>,
  pub force: bool,
  pub quiet: bool,
  pub dry_run: bool,
}

pub fn run(args: ImportArgs) -> Result<()> {
  let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;

  // --- Resolve destination ---
  let dest_path = match &args.dest {
    Some(d) => {
      let expanded = expand_home(d);
      if expanded.is_absolute() {
        expanded
      } else {
        std::env::current_dir()?.join(expanded)
      }
    }
    None => home.join(".dotfiles"),
  };

  // --- Validate destination is inside HOME ---
  if !dest_path.starts_with(&home) {
    bail!(
      "Destination must be inside your HOME directory.\nHOME: {}\nDEST: {}",
      home.display(),
      dest_path.display()
    );
  }

  // --- Check existing config ---
  let config_path = Config::config_path();
  if config_path.exists() && !args.force {
    let existing = Config::load()?;
    let existing_repo = existing.repo_path();
    if existing_repo != dest_path {
      bail!(
        "config.toml already points to a different repo: {}\nUse --force to overwrite.",
        existing_repo.display()
      );
    }
  }

  // --- Check destination already exists ---
  if dest_path.exists() && !args.dry_run {
    bail!(
      "Destination already exists: {}\nRemove it first or choose a different path.",
      dest_path.display()
    );
  }

  // --- Dry run ---
  if args.dry_run {
    info(&format!(
      "Would clone {} → {}",
      args.url,
      dest_path.display()
    ));
    info(&format!(
      "Would write config.toml with repo = \"{}\"",
      dest_path.display()
    ));
    info("Would run: tildr apply");
    return Ok(());
  }

  // --- Clone ---
  if !args.quiet {
    info(&format!("Cloning {} → {}", args.url, dest_path.display()));
  }

  let status = Command::new("git")
    .args(["clone", &args.url, &dest_path.to_string_lossy()])
    .status()?;

  if !status.success() {
    bail!("git clone failed");
  }

  if !args.quiet {
    success(&format!("Cloned into {}", dest_path.display()));
  }

  // --- Write config.toml ---
  let repo_str = {
    if let Ok(rel) = dest_path.strip_prefix(&home) {
      format!("~/{}", rel.display())
    } else {
      dest_path.display().to_string()
    }
  };

  let mut config = if config_path.exists() {
    Config::load()?
  } else {
    Config::default()
  };

  config.core.repo = repo_str.clone();
  config.save()?;

  if !args.quiet {
    success(&format!("Config written: repo = \"{}\"", repo_str));
  }

  // --- Apply ---
  if !args.quiet {
    info("Applying symlinks...");
  }

  let ctx = tildr_core::context::Context::load()?;

  crate::apply::run(
    &ctx,
    crate::apply::ApplyArgs {
      dry_run: false,
      force: false,
      verbose: false,
      quiet: args.quiet,
    },
  )?;

  let manifest = EncryptManifest::new(&ctx.repo_path);
  if manifest.exists() && GpgIntegration::new(&ctx.repo_path).bundle_path().exists() {
    if !detect_gpg_available() {
      warn(
        "gpg not found — skipping decryption of sensitive files. Install gnupg and run: tildr secret decrypt",
      );
    } else {
      if !args.quiet {
        info("Encrypted files detected — decrypting...");
      }
      let gpg = GpgIntegration::new(&ctx.repo_path);
      gpg.decrypt(&ctx.home_path)?;
      if !args.quiet {
        success("Decrypted sensitive files");
      }
    }
  }

  if !args.quiet {
    success("Import complete.");
  }

  Ok(())
}
