pub mod commands;
pub mod completions;
pub mod parser;
pub use commands::CliCommands;
pub use parser::{Cli, parse};
