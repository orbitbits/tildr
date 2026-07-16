use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage profiles for machine-specific dotfile variants",
  after_help = "\
EXAMPLES:
  tildr profile create work --description 'Work environment'
  tildr profile add work --file .bashrc --variant profiles/work/.bashrc
  tildr profile list
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
  /// Add a file variant to a profile
  Add {
    /// Profile name
    name: String,
    /// Original file (relative path, e.g. .bashrc)
    #[arg(long)]
    file: String,
    /// Variant path (relative path, e.g. profiles/work/.bashrc)
    #[arg(long)]
    variant: String,
  },
  /// Remove a file variant from a profile
  Remove {
    /// Profile name
    name: String,
    /// Original file (relative path)
    #[arg(long)]
    file: String,
  },
  /// List all available profiles
  List,
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
