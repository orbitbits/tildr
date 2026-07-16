use super::{CliCommands, info::CliInfoMode, secret::CliSecretMode};
use crate::commands::{
  exclude::CliExcludeMode, git::CliGitMode, group::CliGroupMode, profile::CliProfileMode,
  repo::CliRepoMode,
};
use tildr_domain::{
  Commands, ExcludeMode, GitMode, GroupMode, InfoMode, ProfileMode, RepoMode, SecretMode,
};

impl From<CliCommands> for Commands {
  fn from(value: CliCommands) -> Self {
    match value {
      CliCommands::Init(cmd) => Commands::Init {
        repo: cmd.repo,
        no_git: cmd.no_git,
        force: cmd.force,
        quiet: cmd.quiet,
      },

      CliCommands::Add(cmd) => Commands::Add {
        paths: cmd.paths,
        force: cmd.force,
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
        nolink: cmd.nolink,
      },

      CliCommands::Restore(cmd) => Commands::Restore {
        targets: cmd.targets,
        all: cmd.all,
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
        force: cmd.force,
      },

      CliCommands::Unlink(cmd) => Commands::Unlink {
        targets: cmd.targets,
        all: cmd.all,
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
        force: cmd.force,
      },

      CliCommands::List(cmd) => Commands::List {
        tree: cmd.tree,
        long: cmd.long,
        export: cmd.export,
        import: cmd.import,
        less: cmd.less,
      },

      CliCommands::Apply(cmd) => Commands::Apply {
        dry_run: cmd.dry_run,
        force: cmd.force,
        verbose: cmd.verbose,
        quiet: cmd.quiet,
      },

      CliCommands::Repo(cmd) => Commands::Repo {
        mode: cmd.mode.into(),
      },
      CliCommands::Cat(cmd) => Commands::Cat {
        target: cmd.target,
        less: cmd.less,
      },

      CliCommands::Completions(cmd) => Commands::Completions {
        shell: cmd.shell.to_string(),
      },

      CliCommands::Status(cmd) => Commands::Status {
        json: cmd.json,
        counter: cmd.counter,
        less: cmd.less,
      },

      CliCommands::Doctor(_) => Commands::Doctor,

      CliCommands::Edit(cmd) => Commands::Edit { target: cmd.target },

      CliCommands::Secret(cmd) => Commands::Secret {
        mode: match cmd.mode {
          CliSecretMode::Add { file } => SecretMode::Add { file },
          CliSecretMode::Remove { file } => SecretMode::Remove { file },
          CliSecretMode::List => SecretMode::List,
          CliSecretMode::Encrypt => SecretMode::Encrypt,
          CliSecretMode::Decrypt => SecretMode::Decrypt,
        },
      },

      CliCommands::Info(cmd) => Commands::Info {
        mode: cmd.mode.into(),
      },

      CliCommands::Git(cmd) => Commands::Git {
        mode: cmd.mode.into(),
      },

      CliCommands::Sync(cmd) => Commands::Sync {
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
        force: cmd.force,
      },
      CliCommands::Del(cmd) => Commands::Del {
        target: cmd.target,
        all: cmd.all,
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
        force: cmd.force,
        purge: cmd.purge,
      },
      CliCommands::Import(cmd) => Commands::Import {
        url: cmd.url,
        dest: cmd.dest,
        force: cmd.force,
        quiet: cmd.quiet,
        dry_run: cmd.dry_run,
      },
      CliCommands::Mv(cmd) => Commands::Mv {
        source: cmd.source,
        dest: cmd.dest,
        dry_run: cmd.dry_run,
        quiet: cmd.quiet,
      },
      CliCommands::Exclude(cmd) => Commands::Exclude {
        mode: match cmd.mode {
          CliExcludeMode::Add { pattern } => ExcludeMode::Add { pattern },
          CliExcludeMode::Remove { pattern } => ExcludeMode::Remove { pattern },
          CliExcludeMode::List => ExcludeMode::List,
        },
      },
      CliCommands::Open(_) => Commands::Open,
      CliCommands::Stats(_) => Commands::Stats,
      CliCommands::Backup(cmd) => Commands::Backup { output: cmd.output },
      CliCommands::Suggest(_) => Commands::Suggest,
      CliCommands::Snapshot(cmd) => Commands::Snapshot { output: cmd.output },
      CliCommands::Group(cmd) => Commands::Group {
        mode: match cmd.mode {
          CliGroupMode::Create { name, files } => GroupMode::Create { name, files },
          CliGroupMode::Add { name, files } => GroupMode::Add { name, files },
          CliGroupMode::Remove { name, files } => GroupMode::Remove { name, files },
          CliGroupMode::Delete { name } => GroupMode::Delete { name },
          CliGroupMode::List => GroupMode::List,
          CliGroupMode::Apply { name } => GroupMode::Apply { name },
          CliGroupMode::Unlink { name } => GroupMode::Unlink { name },
        },
      },
      CliCommands::Profile(cmd) => Commands::Profile {
        mode: match cmd.mode {
          CliProfileMode::List => ProfileMode::List,
          CliProfileMode::Set { name } => ProfileMode::Set { name },
          CliProfileMode::Unset => ProfileMode::Unset,
          CliProfileMode::Current => ProfileMode::Current,
        },
      },
    }
  }
}

impl From<CliInfoMode> for InfoMode {
  fn from(value: CliInfoMode) -> Self {
    match value {
      CliInfoMode::License => InfoMode::License,
      CliInfoMode::Credits => InfoMode::Credits,
    }
  }
}

impl From<CliGitMode> for GitMode {
  fn from(value: CliGitMode) -> Self {
    match value {
      CliGitMode::Status => GitMode::Status,
    }
  }
}

impl From<CliRepoMode> for RepoMode {
  fn from(value: CliRepoMode) -> Self {
    match value {
      CliRepoMode::Path => RepoMode::Path,
    }
  }
}
