mod git;
mod output;
pub mod scenario;

use self::git::RepoGit;
use self::output::{print_conflict_message, print_dry_run};
use self::scenario::{MergeCheck, SyncScenario, classify_sync_scenario, plural};
use anyhow::{Result, bail};
use tildr_core::{constants::APP_NAME, context::Context};
use tildr_crypto::{EncryptManifest, GpgIntegration, detect_gpg_available};
use tildr_git::GitIntegration;
use tildr_ui::{info, success};

pub struct SyncArgs {
  pub dry_run: bool,
  pub quiet: bool,
  pub force: bool,
}

pub fn run(ctx: &Context, args: SyncArgs) -> Result<()> {
  if ctx.config.git.enable == Some(false) {
    bail!("Git operations are disabled in config");
  }

  if !ctx.config.git.available {
    bail!(
      "Git is not available. Run `{} init` again after installing git",
      APP_NAME
    );
  }

  if !ctx.repo_path.exists() {
    bail!("Repository not found: {}", ctx.repo_path.display());
  }

  let git = RepoGit::new(&ctx.repo_path);
  let branch = git.current_branch()?;
  let tracking = git.tracking_branch(&branch)?;

  git.fetch(&tracking.remote)?;

  let remote_ahead = git.count_commits(&format!("HEAD..{}", tracking.upstream_ref()))?;
  let local_ahead = git.count_commits(&format!("{}..HEAD", tracking.upstream_ref()))?;

  let scenario = classify_sync_scenario(local_ahead, remote_ahead);

  if args.dry_run {
    print_dry_run(ctx, &tracking, scenario, args.quiet);
    return Ok(());
  }

  match scenario {
    SyncScenario::UpToDate => {
      if !args.quiet {
        success("Already up to date");
      }
    }
    SyncScenario::PushOnly { local_ahead } => {
      re_encrypt_before_push(ctx)?;
      git.push(&tracking, args.force)?;

      if !args.quiet {
        success(&format!(
          "Pushed {} commit{} to remote",
          local_ahead,
          plural(local_ahead)
        ));
      }
    }
    SyncScenario::PullOnly { remote_ahead } => {
      git.fast_forward_merge(&tracking.upstream_ref())?;

      if !args.quiet {
        success(&format!(
          "Pulled {} commit{} from remote",
          remote_ahead,
          plural(remote_ahead)
        ));
      }
    }
    SyncScenario::Diverged {
      local_ahead: _,
      remote_ahead,
    } => match match git.simulate_merge(&tracking.upstream_ref()) {
      Ok(result) => Ok(result),
      Err(err) => {
        let _ = git.merge_abort();
        Err(err)
      }
    }? {
      MergeCheck::Clean => {
        if let Err(err) = git.commit_merge(&tracking) {
          let _ = git.merge_abort();
          return Err(err);
        }

        let pushed = git.count_commits(&format!("{}..HEAD", tracking.upstream_ref()))?;
        re_encrypt_before_push(ctx)?;
        git.push(&tracking, args.force)?;

        if !args.quiet {
          success(&format!(
            "Pulled {} commit{} from remote",
            remote_ahead,
            plural(remote_ahead)
          ));
          success(&format!(
            "Pushed {} commit{} to remote",
            pushed,
            plural(pushed)
          ));
        }
      }
      MergeCheck::Conflicted(files) => {
        let _ = git.merge_abort();
        print_conflict_message(ctx, &files);
        bail!("Sync failed due to merge conflicts");
      }
    },
  }

  Ok(())
}

fn re_encrypt_before_push(ctx: &Context) -> Result<()> {
  let manifest = EncryptManifest::new(&ctx.repo_path);
  if manifest.exists() && detect_gpg_available() {
    let entries = manifest.entries()?;
    if !entries.is_empty() {
      // Filter out files that don't exist in HOME
      let available: Vec<String> = entries
        .iter()
        .filter(|e| ctx.home_path.join(e).exists())
        .cloned()
        .collect();
      let missing: Vec<&String> = entries
        .iter()
        .filter(|e| !ctx.home_path.join(e).exists())
        .collect();
      if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        info(&format!(
          "Skipping {} file(s) not found in HOME: {}",
          missing.len(),
          names.join(", ")
        ));
      }
      if available.is_empty() {
        return Ok(());
      }
      let gpg = GpgIntegration::new(&ctx.repo_path);
      crate::secret::encrypt_bundle(ctx, &gpg, &available)?;
      let git_inner = GitIntegration::new(ctx.repo_path.clone());
      let _ = git_inner.auto_commit(&format!("{}: secret: re-encrypt before sync", APP_NAME));
    }
  }
  Ok(())
}
