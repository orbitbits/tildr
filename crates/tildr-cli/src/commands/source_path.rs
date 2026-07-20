use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Print the repository source path for a managed HOME file",
  after_help = "\
EXAMPLES:
  tildr source-path ~/.bash_profile
  tildr source-path .bashrc --profile linux\n"
)]
pub struct Command {
  /// Managed HOME file path
  pub target: String,

  /// Resolve the file from a specific profile, or no-profile for shared files
  #[arg(long, value_name = "NAME")]
  pub profile: Option<String>,
}
