use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage profiles for machine-specific dotfile variants",
  after_help = "\
EXAMPLES:
  tildr profile create work --description 'Work environment'
  tildr profile add default --files .bashrc --to work
  tildr profile mv work --to default
  tildr profile mv default -f .bashrc --to work
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
  /// Copy files between a profile and the default (bidirectional)
  Add {
    /// Files to copy (omit to copy all)
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,
    /// Source (profile name or "default")
    from: String,
    /// Destination (profile name or "default")
    #[arg(short, long)]
    to: String,
  },
  /// Move files between a profile and the default (bidirectional)
  Mv {
    /// Files to move (omit to move all)
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,
    /// Source (profile name or "default")
    from: String,
    /// Destination (profile name or "default")
    #[arg(short, long)]
    to: String,
  },
  /// Delete a profile entirely (restores files to default)
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
  /// Set the active profile
  Set {
    /// Profile name
    name: String,
  },
  /// Unset the active profile (revert to default)
  Unset,
  /// Show the currently active profile
  Current,
}
