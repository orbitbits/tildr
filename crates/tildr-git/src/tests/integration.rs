use crate::integration::{
  GitStatusIssue, GitStatusIssueKind, detect_git_available_with, git_locator_command,
  parse_status_issues,
};
use std::cell::RefCell;

#[test]
fn detection_uses_which_on_unix() {
  assert_eq!(git_locator_command("linux"), ("which", "git"));
  assert_eq!(git_locator_command("macos"), ("which", "git"));
}

#[test]
fn detection_uses_where_on_windows() {
  assert_eq!(git_locator_command("windows"), ("where", "git"));
}

#[test]
fn detection_returns_runner_status() {
  let seen = RefCell::new(Vec::new());

  let detected = detect_git_available_with("linux", |program, executable| {
    seen
      .borrow_mut()
      .push((program.to_string(), executable.to_string()));
    Ok(true)
  });

  assert!(detected);
  assert_eq!(
    seen.into_inner(),
    vec![("which".to_string(), "git".to_string())]
  );
}

#[test]
fn detection_returns_false_on_runner_error() {
  let detected = detect_git_available_with("windows", |_program, _executable| {
    Err(std::io::Error::other("missing"))
  });

  assert!(!detected);
}

#[test]
fn parse_status_issues_detects_untracked_and_uncommitted_files() {
  let issues = parse_status_issues("?? new.txt\n M modified.txt\nA  staged.txt\n");

  assert_eq!(
    issues,
    vec![
      GitStatusIssue {
        kind: GitStatusIssueKind::Untracked,
        path: "new.txt".to_string(),
      },
      GitStatusIssue {
        kind: GitStatusIssueKind::Uncommitted,
        path: "modified.txt".to_string(),
      },
      GitStatusIssue {
        kind: GitStatusIssueKind::Uncommitted,
        path: "staged.txt".to_string(),
      }
    ]
  );
}

#[test]
fn parse_status_issues_ignores_clean_output() {
  assert!(parse_status_issues("").is_empty());
}
