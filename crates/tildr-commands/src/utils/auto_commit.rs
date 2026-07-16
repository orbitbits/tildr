use tildr_core::{constants::APP_NAME, context::Context};
use tildr_git::GitIntegration;

/// Auto-commit changes to the repository if auto-commit is enabled.
pub fn auto_commit(ctx: &Context, msg: &str) {
  if ctx.config.git.auto_commit_enabled() {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}

/// Auto-commit changes to the repository if auto-commit is enabled and not a dry run.
pub fn auto_commit_dry_run(ctx: &Context, msg: &str, dry_run: bool) {
  if ctx.config.git.auto_commit_enabled() && !dry_run {
    let git = GitIntegration::new(ctx.repo_path.clone());
    let _ = git.auto_commit(&format!("{}: {}", APP_NAME, msg));
  }
}
