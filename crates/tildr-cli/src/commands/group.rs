use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage groups of managed files",
  after_help = "\
EXAMPLES:
  tildr group list
  tildr group create dev --files .bashrc .config/nvim
  tildr group add dev --files .tmux.conf
  tildr group rm dev --files .tmux.conf
  tildr group rename dev shell
  tildr group apply dev
  tildr group unlink dev\n"
)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliGroupMode,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliGroupMode {
  /// Create a new group with files
  Create {
    /// Group name
    name: String,
    /// Files or folders to include (relative paths). Folders are expanded recursively.
    #[arg(long, num_args = 1..)]
    files: Vec<String>,
  },
  /// Add files or folders to an existing group
  Add {
    /// Group name
    name: String,
    /// Files or folders to add (relative paths). Folders are expanded recursively.
    /// If omitted, opens a file picker.
    #[arg(long, num_args = 1..)]
    files: Option<Vec<String>>,
  },
  /// Remove files or folders from a group
  #[command(name = "rm")]
  Remove {
    /// Group name
    name: String,
    /// Files or folders to remove (relative paths). Folders remove all entries inside recursively.
    #[arg(long, num_args = 1..)]
    files: Vec<String>,
  },
  /// Delete a group
  Delete {
    /// Group name
    name: String,
  },
  /// Rename a group
  Rename {
    /// Current group name
    from: Option<String>,
    /// New group name
    to: Option<String>,
  },
  /// List all groups
  List,
  /// Create symlinks for all files in a group
  Apply {
    /// Group name
    name: String,
  },
  /// Remove symlinks for all files in a group
  Unlink {
    /// Group name
    name: String,
  },
}
