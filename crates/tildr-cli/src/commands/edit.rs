use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Edit a managed file from the Tildr repository",
  after_help = "\
EXAMPLES:
  tildr edit
  tildr edit .config/nvim/init.vim\n"
)]
pub struct Command {
  /// File to edit. If not provided, an interactive picker will be shown
  pub target: Option<String>,
}
