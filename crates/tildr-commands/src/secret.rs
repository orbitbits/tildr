use anyhow::{Result, bail};
use std::process::Command;
use tildr_core::{CryptoMode, config::Config, constants::APP_NAME, context::Context};
use tildr_crypto::{EncryptManifest, GpgIntegration, detect_gpg_available};
use tildr_domain::SecretMode;
use tildr_fs::paths::resolve_home_path;
use tildr_git::GitIntegration;
use tildr_ui::{color::Colorize, info, prompt::MinimalTheme, success};

pub fn run(ctx: &Context, mode: SecretMode) -> Result<()> {
  if !detect_gpg_available() {
    bail!("gpg not found in PATH. Install gnupg first.");
  }

  let manifest = EncryptManifest::new(&ctx.repo_path);
  let gpg = GpgIntegration::new(&ctx.repo_path);

  match mode {
    SecretMode::Add { file } => {
      let source = resolve_home_path(&file, &ctx.home_path);
      if !source.exists() {
        bail!(
          "File not found: {} \nTry using tilder secret decrypt first to extract the files.",
          source.display()
        );
      }

      let relative = source
        .strip_prefix(&ctx.home_path)
        .map_err(|_| anyhow::anyhow!("File must be inside HOME directory"))?
        .to_string_lossy()
        .to_string();

      manifest.add(&relative)?;
      info(&format!("Added to .tildr/encrypted-items: {}", relative));

      add_to_gitignore(&ctx.repo_path, &relative)?;
      git_untrack_if_tracked(&ctx.repo_path, &relative)?;

      let entries = manifest.entries()?;
      encrypt_bundle(ctx, &gpg, &entries)?;
      success("Encrypted bundle updated");

      auto_commit(ctx, &format!("secret: add {}", relative));
    }

    SecretMode::List => {
      let entries = manifest.entries()?;
      if entries.is_empty() {
        info("No sensitive files registered. Use: tildr secret add <file>");
        return Ok(());
      }
      println!();
      println!("{}", "Sensitive files".cyan());
      println!("{}", "---------------".cyan());
      for entry in &entries {
        println!("  {}", entry);
      }
      println!();
    }

    SecretMode::Remove { file } => {
      let removed = manifest.remove(&file)?;
      if !removed {
        bail!("File not in .tildr/encrypted-items: {}", file);
      }
      info(&format!("Removed from .tildr/encrypted-items: {}", file));

      let entries = manifest.entries()?;
      if entries.is_empty() {
        let bundle = gpg.bundle_path();
        if bundle.exists() {
          std::fs::remove_file(&bundle)?;
          info("Encrypted bundle removed (no more entries)");
        }
      } else {
        encrypt_bundle(ctx, &gpg, &entries)?;
        success("Encrypted bundle updated");
      }

      auto_commit(ctx, &format!("secret: remove {}", file));
    }

    SecretMode::Encrypt => {
      let entries = manifest.entries()?;
      if entries.is_empty() {
        bail!("No files registered. Use: tildr secret add <file>");
      }
      encrypt_bundle(ctx, &gpg, &entries)?;
      success("Bundle encrypted successfully");
      auto_commit(ctx, "secret: re-encrypt");
    }

    SecretMode::Decrypt => {
      if !gpg.bundle_path().exists() {
        bail!("No encrypted bundle found. Use: tildr secret add <file>");
      }
      info("Decrypting bundle...");
      gpg.decrypt(&ctx.home_path)?;
      success("Files restored to HOME");
    }
  }

  Ok(())
}

pub fn encrypt_bundle(ctx: &Context, gpg: &GpgIntegration, entries: &[String]) -> Result<()> {
  match ctx.config.crypto.mode {
    CryptoMode::Symmetric => {
      gpg.encrypt_symmetric(entries, &ctx.home_path)?;
    }
    CryptoMode::Asymmetric => {
      let key = resolve_gpg_key(ctx, gpg)?;
      gpg.encrypt_asymmetric(entries, &ctx.home_path, &key)?;
    }
  }

  Ok(())
}

fn resolve_gpg_key(ctx: &Context, gpg: &GpgIntegration) -> Result<String> {
  if !ctx.config.crypto.gpg_key.is_empty() {
    return Ok(ctx.config.crypto.gpg_key.clone());
  }

  let keys = gpg.list_secret_keys()?;

  if keys.is_empty() {
    bail!(
      "No GPG secret keys found.\nCreate one with: gpg --gen-key\nThen set [crypto].gpg_key in config.toml"
    );
  }

  if keys.len() == 1 {
    let (key_id, uid) = &keys[0];
    info(&format!("Using GPG key: {} ({})", uid, key_id));
    save_gpg_key_to_config(key_id)?;
    return Ok(key_id.clone());
  }

  let items: Vec<String> = keys
    .iter()
    .map(|(id, uid)| format!("{} ({})", uid, id))
    .collect();

  println!();
  let selection = dialoguer::Select::with_theme(&MinimalTheme)
    .with_prompt("Select GPG key to use")
    .items(&items)
    .default(0)
    .interact_opt()?;

  match selection {
    Some(i) => {
      let (key_id, _) = &keys[i];
      save_gpg_key_to_config(key_id)?;
      Ok(key_id.clone())
    }
    None => std::process::exit(130),
  }
}

fn save_gpg_key_to_config(key_id: &str) -> Result<()> {
  let mut config = Config::load()?;
  config.crypto.gpg_key = key_id.to_string();
  config.save()?;
  info(&format!("GPG key saved to config: {}", key_id));
  Ok(())
}

fn add_to_gitignore(repo_path: &std::path::Path, relative: &str) -> Result<()> {
  let gitignore = repo_path.join(".gitignore");
  let mut content = if gitignore.exists() {
    std::fs::read_to_string(&gitignore)?
  } else {
    String::new()
  };

  let line = relative.to_string();
  if !content.lines().any(|l| l.trim() == line) {
    if !content.ends_with('\n') && !content.is_empty() {
      content.push('\n');
    }
    content.push_str(&line);
    content.push('\n');
    std::fs::write(&gitignore, content)?;
  }

  Ok(())
}

fn git_untrack_if_tracked(repo_path: &std::path::Path, relative: &str) -> Result<()> {
  let git_dir = repo_path.join(".git");
  let git_dir_arg = format!("--git-dir={}", git_dir.display());
  let work_tree_arg = format!("--work-tree={}", repo_path.display());

  let output = Command::new("git")
    .args([
      &git_dir_arg,
      &work_tree_arg,
      "ls-files",
      "--error-unmatch",
      relative,
    ])
    .output()?;

  if output.status.success() {
    let rm_output = Command::new("git")
      .args([&git_dir_arg, &work_tree_arg, "rm", "--cached", relative])
      .output()?;

    if !rm_output.status.success() {
      bail!(
        "git rm --cached failed: {}",
        String::from_utf8_lossy(&rm_output.stderr)
      );
    }
  }

  Ok(())
}

fn auto_commit(ctx: &Context, msg: &str) {
  if ctx.config.git.auto_commit_enabled() {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}
