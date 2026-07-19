use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage profile variants and switch dotfiles between machines",
  long_about = "\
Manage profile variants for machine-specific dotfiles.

Files in common/ are shared by every machine. A named profile such as
linux, work, or laptop can override any common file. When you run
`tildr profile set <name>`, Tildr saves the active profile and immediately
relinks HOME so matching files point at profiles/<name>/ while the rest keep
using common/.",
  after_help = "\
EXAMPLES:
  # First-time migration: move repo-root dotfiles and legacy profiles/common/ into common/
  tildr profile migrate --dry-run
  tildr profile migrate

  # Create a machine profile and move selected common files into it
  tildr profile create work --description 'Work environment'
  tildr profile mv common -f .bashrc .ssh/config --to work

  # Switch profile and relink HOME immediately
  tildr profile set work
  tildr status

  # Copy or move variants between profiles
  tildr profile add work -f .bashrc --to laptop
  tildr profile mv work --to common

  tildr profile del work
  tildr profile list
  tildr profile list --long
  tildr profile list work --long
  tildr profile list --less
  tildr profile set work
  tildr profile current
  tildr profile unset\n"
)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliProfileMode,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliProfileMode {
  /// Create a new profile
  Create {
    /// Profile name
    name: String,
    /// Optional description
    #[arg(long)]
    description: Option<String>,
  },
  /// Copy files between common files and profiles
  Add {
    /// Files to copy (omit to copy all)
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,
    /// Source ("common" or profile name)
    from: String,
    /// Destination ("common" or profile name)
    #[arg(short, long)]
    to: String,
  },
  /// Move files between common files and profiles
  Mv {
    /// Files to move (omit to move all)
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,
    /// Source ("common" or profile name)
    from: String,
    /// Destination ("common" or profile name)
    #[arg(short, long)]
    to: String,
  },
  /// Delete a profile entirely (restores files to common)
  #[command(name = "del")]
  Delete {
    /// Profile name
    name: String,
  },
  /// Rename a profile
  Rename {
    /// Current profile name
    from: String,
    /// New profile name
    to: String,
  },
  /// List all available profiles
  List {
    /// Show files in each profile
    #[arg(short, long)]
    long: bool,
    /// Page output through less
    #[arg(long)]
    less: bool,
    /// Profile name to show
    name: Option<String>,
  },
  /// Set the active profile and relink HOME
  Set {
    /// Profile name
    name: String,
  },
  /// Unset the active profile and relink HOME to common files
  Unset,
  /// Show the currently active profile
  Current,
  /// Move repo-root dotfiles into common/
  #[command(
    long_about = "\
Move dotfiles stored directly at the repository root into common/.

Use this when upgrading an older Tildr repository to the profile layout. The
command preserves relative paths, so `.bashrc` becomes `common/.bashrc` and
`.config/nvim/init.lua` becomes `common/.config/nvim/init.lua`.

Legacy files under `profiles/common/` are also moved into `common/` when the
target path does not already exist.

Tildr internals are left in place: `.tildr/`, `.git/`, and `profiles/` are not
moved. Run with --dry-run first to preview every move.",
    after_help = "\
EXAMPLES:
  tildr profile migrate --dry-run
  tildr profile migrate
  tildr profile list --long
  tildr status\n"
  )]
  Migrate {
    /// Show what would be done without making changes
    #[arg(short, long)]
    dry_run: bool,
  },
}
