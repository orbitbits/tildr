#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn compact_issue_summary_skips_zero_counts() {
    assert_eq!(
      compact_issue_summary(&[(0, "Broken symlink"), (2, "Missing link")]),
      "2 Missing link"
    );
  }

  #[test]
  fn compact_issue_summary_joins_multiple_issue_types() {
    assert_eq!(
      compact_issue_summary(&[(1, "Broken symlink"), (2, "Missing link")]),
      "1 Broken symlink, 2 Missing link"
    );
  }
}
