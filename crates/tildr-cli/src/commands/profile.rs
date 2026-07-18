use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage profiles for machine-specific dotfile variants",
  after_help = "\
EXAMPLES:
  tildr profile create work --description 'Work environment'
  tildr profile add work --files .bashrc .ssh/config
  tildr profile rm work --files .bashrc
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
  /// Add files to a profile (copies them to profiles/<name>/)
  Add {
    /// Profile name
    name: String,
    /// Files to add (relative paths). Folders are expanded recursively.
    #[arg(long, num_args = 1..)]
    files: Vec<String>,
  },
  /// Remove files from a profile
  #[command(name = "rm")]
  Remove {
    /// Profile name
    name: String,
    /// Files to remove (relative paths)
    #[arg(long, num_args = 1..)]
    files: Vec<String>,
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
