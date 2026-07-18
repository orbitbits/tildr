pub mod ignore;
pub mod manager;
pub mod scanner;

pub use manager::RepoManager;
pub use scanner::{ManagedEntry, scatildr_repo};

#[cfg(test)]
mod tests;
