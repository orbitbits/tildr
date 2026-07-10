pub(crate) fn compact_issue_summary(counts: &[(usize, &str)]) -> String {
  counts
    .iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{count} {label}"))
    .collect::<Vec<_>>()
    .join(", ")
}
