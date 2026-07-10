use std::fs;

use anyhow::Result;
use tildr_core::{config::Config, constants::APP_NAME};
use tildr_fs::paths::expand_home;
use tildr_git::{GitIntegration, detect_git_available};
use tildr_ui::{color::Colorize, info, success, warn};
use tildr_utils::traits::Capitalize;

// ==========================================================
// TODO(link-strategy): Future support for multiple link strategies
// ==========================================================
//
// Currently Tildr only supports "symlink" strategy.
// In the future we may support:
//   - "symlink"  → default, lightweight, Unix-native
//   - "copy"     → safer, portable, no broken links
//   - (maybe) "hardlink"
//
// ----------------------------------------------------------
// DESIGN DECISION
// ----------------------------------------------------------
//
// The chosen strategy MUST be IMMUTABLE after repository initialization.
//
// Why:
// - Mixing strategies causes undefined state:
//   - Some files as symlink, others as copy
//   - Difficult to detect real source of truth
//   - Breaks `apply`, `rm`, `status` logic
//
// Therefore:
//
// Changing link.strategy is NOT supported
// User must reinitialize the repo if they want a different strategy
//
// UX:
//
//   "Changing link strategy is not supported.
//    Please run `tildr init --force` to reinitialize the repository."
//
// ----------------------------------------------------------
// META FILE (.tildr/meta.toml)
// ----------------------------------------------------------
//
// We must persist repository-level metadata in:
//
//   <repo>/.tildr/meta.toml
//
// This file defines how the repo was created.
//
// Example:
//
// [meta]
// version = 1
// created_at = "2026-05-02T12:00:00Z"
//
// [link]
// strategy = "symlink"
//
// ----------------------------------------------------------
// VALIDATION RULES (IMPORTANT)
// ----------------------------------------------------------
//
// On Context::load():
//
// 1. Load user config (~/.config/tildr/config.toml)
// 2. Load repo meta (.tildr/meta.toml)
//
// 3. Validate:
//
//    if config.link.strategy != meta.link.strategy {
//        ERROR:
//        "Link strategy mismatch.
//         Repo was initialized with 'symlink' but config is 'copy'.
//         Changing strategy is not supported."
//    }
//
// ----------------------------------------------------------
// INIT BEHAVIOR
// ----------------------------------------------------------
//
// On `tildr init`:
//
// - Create .tildr/meta.toml if not exists
// - Persist:
//
//     meta.version = 1
//     link.strategy = config.link.strategy
//
// ----------------------------------------------------------
// FUTURE MIGRATION (versioning ready)
// ----------------------------------------------------------
//
// When meta.version changes:
//
// - Implement migration layer:
//
//     match meta.version {
//         1 => OK,
//         _ => attempt migration or fail with instruction
//     }
//
// ----------------------------------------------------------
// IMPLEMENTATION NOTES
// ----------------------------------------------------------
//
// - Create struct:
//
//     struct RepoMeta {
//         version: u32,
//         link: LinkMeta,
//     }
//
//     struct LinkMeta {
//         strategy: String,
//     }
//
// - Always read BEFORE any operation (add/apply/rm)
//
// - Never silently fallback
// - Always fail explicitly
//
// ----------------------------------------------------------
// END TODO
// ==========================================================

pub struct InitArgs {
  pub repo: Option<String>,
  pub no_git: bool,
  pub force: bool,
  pub quiet: bool,
}

pub fn run(args: InitArgs) -> Result<()> {
  let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

  let repo_path = match args.repo {
    Some(r) => {
      let expanded = expand_home(&r);
      if expanded.is_absolute() {
        expanded
      } else {
        std::env::current_dir()?.join(expanded)
      }
    }
    // If I don't use the --repo flag, the repository path will automatically be ~/.dotfiles.
    None => home.clone().join(".dotfiles"),
  };

  if repo_path == home {
    anyhow::bail!(
      "Repository cannot be the HOME directory.\n\
     Please specify a subdirectory (e.g. ~/.dotfiles)"
    );
  }

  if !repo_path.starts_with(&home) {
    anyhow::bail!(
      "Repository must be inside your HOME directory.\n\
     HOME: {}\n\
     REPO: {}\n\
     Hint: use something like ~/.dotfiles",
      home.display(),
      repo_path.display()
    );
  }

  let repo_existed = repo_path.exists();
  fs::create_dir_all(&repo_path)?;

  let config_path = Config::config_path();
  let config_existed = config_path.exists();

  if config_existed && !args.force && !args.quiet {
    info(&format!(
      "{} is already initialized. Refreshing git configuration.",
      APP_NAME.to_string().capitalize()
    ));
  }

  if !args.quiet {
    success(&format!(
      "{} repo at {}",
      if repo_existed {
        "Updated".yellow()
      } else {
        "Created".green()
      },
      repo_path.display()
    ));
  }

  let repo_str = if let Some(home) = dirs::home_dir() {
    if let Ok(rel) = repo_path.strip_prefix(&home) {
      format!("~/{}", rel.display())
    } else {
      repo_path.display().to_string()
    }
  } else {
    repo_path.display().to_string()
  };

  let git_available = detect_git_available();
  let mut config = if config_path.exists() {
    Config::load()?
  } else {
    Config::default()
  };
  config.core.repo = repo_str;
  config.git.available = git_available;
  config.save()?;
  if !args.quiet {
    success(&format!(
      "{} config at {}",
      if config_existed {
        "Updated".yellow()
      } else {
        "Created".green()
      },
      config_path.display()
    ));
  }

  if !args.no_git && config.git.operations_enabled() {
    let git = GitIntegration::new(repo_path.clone());
    if !git.is_git_repo() {
      match git.init() {
        Ok(_) => success("Initialized git repository"),
        Err(e) => warn(&format!("Could not initialize git: {}", e)),
      }
    }
  } else if !git_available && !args.quiet {
    info(&format!(
      "Git was not found in PATH. You can run `{}` again after installing git to enable repository integration.",
      APP_NAME
    ));
  }

  let info_format = format!(
    "{} initialized. Run `{} add <file>` to start managing files.",
    APP_NAME.to_string().capitalize(),
    APP_NAME
  );

  if !args.quiet {
    info(&info_format);
  }

  Ok(())
}
