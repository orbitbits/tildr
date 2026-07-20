mod integration;
pub use integration::{GitIntegration, GitStatusIssue, GitStatusIssueKind, detect_git_available};

#[cfg(test)]
mod tests;
