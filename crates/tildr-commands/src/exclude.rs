use anyhow::Result;
use tildr_core::context::Context;
use tildr_domain::ExcludeMode;

use crate::utils::auto_commit::auto_commit;
use crate::utils::tildrignore;

pub fn run(ctx: &Context, mode: ExcludeMode) -> Result<()> {
  match mode {
    ExcludeMode::Add { pattern } => {
      tildrignore::append(&ctx.repo_path, &pattern)?;
      auto_commit(ctx, &format!("exclude add {}", pattern));
    }
    ExcludeMode::Remove { pattern } => {
      tildrignore::remove(&ctx.repo_path, &pattern)?;
      auto_commit(ctx, &format!("exclude remove {}", pattern));
    }
    ExcludeMode::List => {
      let patterns = tildrignore::list(&ctx.repo_path)?;
      if patterns.is_empty() {
        println!("No patterns in .tildrignore");
      } else {
        for pattern in &patterns {
          println!("{}", pattern);
        }
      }
    }
  }
  Ok(())
}
