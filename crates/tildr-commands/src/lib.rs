mod add;
mod apply;
mod backup;
mod cat;
mod del;
pub mod doctor;
mod edit;
mod exclude;
mod git;
mod group;
mod import;
mod info;
mod init;
mod list;
mod mv;
mod open;
mod profile;
mod repo;
mod restore;
mod secret;
mod snapshot;
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
      profile,
      force,
      dry_run,
      quiet,
      nolink,
    } => add::run(
      ctx,
      add::AddArgs {
        paths: paths.clone(),
        profile: profile.clone(),
        force: *force,
        dry_run: *dry_run,
        quiet: *quiet,
        nolink: *nolink,
      },
    ),

    Commands::Restore {
      targets,
      profile,
      all,
      dry_run,
      quiet,
      force,
    } => restore::run(
      ctx,
      targets.clone(),
      restore::RestoreArgs {
        profile: profile.clone(),
        all: *all,
        dry_run: *dry_run,
        quiet: *quiet,
        force: *force,
      },
    ),

    Commands::Unlink {
      targets,
      profile,
      all,
      dry_run,
      quiet,
      force,
    } => unlink::run(
      ctx,
      targets.clone(),
      *all,
      unlink::UnlinkArgs {
        profile: profile.clone(),
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
      less,
      profile,
    } => list::run(
      ctx,
      list::ListArgs {
        tree: *tree,
        long: *long,
        export: export.clone(),
        import: import.clone(),
        less: *less,
        profile: profile.clone(),
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

    Commands::Cat {
      target,
      less,
      profile,
    } => cat::run(
      ctx,
      cat::CatArgs {
        target: target.clone(),
        less: *less,
        profile: profile.clone(),
      },
    ),

    Commands::Status {
      json,
      counter,
      long,
      less,
      profile,
    } => status::run(
      ctx,
      status::StatusArgs {
        json: *json,
        counter: *counter,
        long: *long,
        less: *less,
        profile: profile.clone(),
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
      profile,
      all,
      dry_run,
      quiet,
      force,
      purge,
    } => del::run(
      ctx,
      target.clone(),
      del::DelArgs {
        profile: profile.clone(),
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
    Commands::Snapshot { output } => snapshot::run(ctx, output),
    Commands::Group { mode } => group::run(ctx, mode),
    Commands::Profile { mode } => profile::run(ctx, mode),
  }
}

#[cfg(test)]
mod tests;
