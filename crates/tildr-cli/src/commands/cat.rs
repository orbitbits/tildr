use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Display the contents of a managed file",
  after_help = "\
EXAMPLES:
  tildr cat
  tildr cat .config/nvim/init.vim
  tildr cat .bashrc --profile linux
  tildr cat .config/nvim/init.vim --less
  tildr cat config\n"
)]
pub struct Command {
  /// Path to a managed file, or a special target like 'config'. If not provided, an interactive picker is shown
  pub target: Option<String>,

  /// View the contents of a managed file in an interactive pager
  #[arg(short, long)]
  pub less: bool,

  /// Read the file from a specific profile instead of using the active one
  #[arg(long, value_name = "NAME")]
  pub profile: Option<String>,
}
