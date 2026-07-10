use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(
  about = "Manage encryption/decryption of sensitive dotfiles",
  after_help = "\
EXAMPLES:
  tildr secret add ~/.ssh/id_rsa
  tildr secret list
  tildr secret encrypt
  tildr secret decrypt\n"
)]
pub struct Command {
  #[command(subcommand)]
  pub mode: CliSecretMode,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliSecretMode {
  /// Register a file as sensitive and encrypt it
  Add {
    /// Path to the sensitive file (absolute or ~/relative)
    file: String,
  },

  /// Unregister a file (does not delete the original)
  Remove {
    /// Relative path as listed in .tildr-encrypt
    file: String,
  },

  /// List all registered sensitive files
  List,

  /// Re-encrypt all registered files into the bundle
  Encrypt,

  /// Decrypt the bundle and restore files to HOME
  Decrypt,
}
