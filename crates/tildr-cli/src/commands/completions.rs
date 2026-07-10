use clap::Args;
use clap_complete::Shell;

#[derive(Args, Debug, Clone)]
#[command(
  about = "Generate shell completion scripts",
  after_help = "\
EXAMPLES:
  tildr completions bash
  tildr completions zsh
  tildr completions fish

INSTALL:
  bash:            tildr completions bash >> ~/.bash_completion
  zsh (Oh My Zsh): tildr completions zsh > ~/.oh-my-zsh/completions/_tildr
  zsh (vanilla):   tildr completions zsh > ~/.zfunc/_tildr
  fish:            tildr completions fish > ~/.config/fish/completions/tildr.fish\n"
)]
pub struct Command {
  /// Shell to generate completions for
  pub shell: Shell,
}
