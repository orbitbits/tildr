mod add;
mod apply;
mod backup;
mod cat;
mod del;
pub mod doctor;
mod edit;
mod exclude;
mod git;
mod import;
mod info;
mod init;
mod list;
mod mv;
mod open;
mod repo;
mod restore;
mod secret;
pub mod stats;
mod status;
mod suggest;
pub mod sync;
mod unlink;
mod utils;

use anyhow::Result;
use tildr_core::context::Context;
use tildr_domain::Commands;

pub fn dispatch(cmd: Commands) -> Result<()> {
  match &cmd {
    Commands::Init {
      repo,
      no_git,
      force,
      quiet,
    } => init::run(init::InitArgs {
      repo: repo.clone(),
      no_git: *no_git,
      force: *force,
      quiet: *quiet,
    }),

    Commands::Completions { shell } => tildr_cli::completions::generate_completions(shell),

    Commands::Import {
      url,
      dest,
      force,
      quiet,
      dry_run,
    } => import::run(import::ImportArgs {
      url: url.clone(),
      dest: dest.clone(),
      force: *force,
      quiet: *quiet,
      dry_run: *dry_run,
    }),

    _ => {
      let ctx = Context::load()?;
      dispatch_with_ctx(&cmd, &ctx)
    }
  }
}

fn dispatch_with_ctx(cmd: &Commands, ctx: &Context) -> Result<()> {
  match cmd {
    Commands::Init { .. } => unreachable!(),
    Commands::Completions { .. } => unreachable!(),
    Commands::Import { .. } => unreachable!(),

    Commands::Add {
      paths,
      force,
      dry_run,
      quiet,
      nolink,
    } => add::run(
      ctx,
      add::AddArgs {
        paths: paths.clone(),
        force: *force,
        dry_run: *dry_run,
        quiet: *quiet,
        nolink: *nolink,
      },
    ),

    Commands::Restore {
      targets,
      all,
      dry_run,
      quiet,
      force,
    } => restore::run(
      ctx,
      targets.clone(),
      restore::RestoreArgs {
        all: *all,
        dry_run: *dry_run,
        quiet: *quiet,
        force: *force,
      },
    ),

    Commands::Unlink {
      targets,
      all,
      dry_run,
      quiet,
      force,
    } => unlink::run(
      ctx,
      targets.clone(),
      *all,
      unlink::UnlinkArgs {
        dry_run: *dry_run,
        quiet: *quiet,
        force: *force,
      },
    ),

    Commands::List {
      tree,
      long,
      export,
      import,
    } => list::run(
      ctx,
      list::ListArgs {
        tree: *tree,
        long: *long,
        export: export.clone(),
        import: import.clone(),
      },
    ),

    Commands::Apply {
      dry_run,
      force,
      verbose,
      quiet,
    } => apply::run(
      ctx,
      apply::ApplyArgs {
        dry_run: *dry_run,
        force: *force,
        verbose: *verbose,
        quiet: *quiet,
      },
    ),

    Commands::Repo { mode } => repo::run(ctx, repo::RepoArgs { mode: mode.clone() }),

    Commands::Cat { target, less } => cat::run(
      ctx,
      cat::CatArgs {
        target: target.clone(),
        less: *less,
      },
    ),

    Commands::Status { json, counter } => status::run(
      ctx,
      status::StatusArgs {
        json: *json,
        counter: *counter,
      },
    ),

    Commands::Doctor => doctor::run(ctx),

    Commands::Edit { target } => edit::run(
      ctx,
      edit::EditArgs {
        target: target.clone(),
      },
    ),
    Commands::Del {
      target,
      all,
      dry_run,
      quiet,
      force,
      purge,
    } => del::run(
      ctx,
      target.clone(),
      del::DelArgs {
        all: *all,
        dry_run: *dry_run,
        quiet: *quiet,
        force: *force,
        purge: *purge,
      },
    ),
    Commands::Git { mode } => git::run(ctx, git::GitArgs { mode: mode.clone() }),
    Commands::Info { mode } => info::run(ctx, info::InfoArgs { mode: mode.clone() }),
    Commands::Sync {
      dry_run,
      quiet,
      force,
    } => sync::run(
      ctx,
      sync::SyncArgs {
        dry_run: *dry_run,
        quiet: *quiet,
        force: *force,
      },
    ),
    Commands::Mv {
      source,
      dest,
      dry_run,
      quiet,
    } => mv::run(
      ctx,
      mv::MvArgs {
        source: source.clone(),
        dest: dest.clone(),
        dry_run: *dry_run,
        quiet: *quiet,
      },
    ),
    Commands::Secret { mode } => secret::run(ctx, mode.clone()),
    Commands::Exclude { mode } => exclude::run(ctx, mode.clone()),
    Commands::Open => open::run(ctx),
    Commands::Stats => stats::run(ctx),
    Commands::Backup { output } => backup::run(ctx, output),
    Commands::Suggest => suggest::run(ctx),
  }
}
