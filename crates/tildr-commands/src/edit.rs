use anyhow::{Result, bail};
use std::process::Command;
use tildr_core::{
  context::Context,
  pick::{self, PickMode},
};

use crate::utils::target::{FileResolution, resolve_logical_file};

pub struct EditArgs {
  pub target: Option<String>,
  pub profile: Option<String>,
}

pub fn run(ctx: &Context, args: EditArgs) -> Result<()> {
  let editor = std::env::var("EDITOR")
    .or_else(|_| std::env::var("VISUAL"))
    .unwrap_or_else(|_| "nano".to_string());

  let target = pick::target(
    ctx,
    args.target,
    true,
    Some("Select a file\n-------------\n"),
    PickMode::Managed,
  )?;
  let path = if target.is_absolute() && target.exists() && !target.starts_with(&ctx.home_path) {
    target
  } else {
    match resolve_logical_file(ctx, &target, args.profile.as_deref())? {
      FileResolution::Found(entry) => entry.repo_path,
      FileResolution::AmbiguousAcrossProfiles(profiles) => {
        bail!(
          "File '{}' exists in multiple profiles: {}. Use --profile <name>, or run `tildr profile set <name>` first.",
          target.display(),
          profiles.join(", ")
        );
      }
      FileResolution::NotManaged => bail!("Target is not managed: {}", target.display()),
    }
  };

  {
    let status = Command::new(&editor).arg(&path).status()?;

    if !status.success() {
      bail!("Editor '{}' exited with an error", editor);
    }
  }

  Ok(())
}
