use super::{CliCommands, info::CliInfoMode, secret::CliSecretMode};
use crate::commands::{exclude::CliExcludeMode, git::CliGitMode, repo::CliRepoMode};
use tildr_domain::{Commands, ExcludeMode, GitMode, InfoMode, RepoMode, SecretMode};

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
