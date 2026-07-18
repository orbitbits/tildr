use crate::status::*;

fn file_status(path: &str, status: &str) -> FileStatus {
  FileStatus {
    path: path.to_string(),
    status: status.to_string(),
  }
}

#[test]
fn counter_all_empty() {
  let result = counter_all(&Vec::new()).unwrap();
  assert_eq!(result.0, 0);
  assert_eq!(result.1, vec![0, 0, 0, 0]);
}

#[test]
fn counter_all_linked_only() {
  let statuses = vec![file_status("a", "linked"), file_status("b", "linked")];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.0, 2);
  assert_eq!(result.1, vec![2, 0, 0, 0]);
}

#[test]
fn counter_all_mixed_statuses() {
  let statuses = vec![
    file_status("a", "linked"),
    file_status("b", "missing_link"),
    file_status("c", "broken_symlink"),
    file_status("d", "not_a_symlink"),
  ];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.0, 4);
  assert_eq!(result.1, vec![1, 1, 1, 1]);
}

#[test]
fn counter_all_unknown_status_is_ignored() {
  let statuses = vec![file_status("a", "unknown")];
  let result = counter_all(&statuses).unwrap();
  assert_eq!(result.1, vec![0, 0, 0, 0]);
}

#[test]
fn file_status_serialization() {
  let fs = FileStatus {
    path: ".bashrc".to_string(),
    status: "linked".to_string(),
  };
  let json = serde_json::to_string(&fs).unwrap();
  assert!(json.contains("\"path\":\".bashrc\""));
  assert!(json.contains("\"status\":\"linked\""));
}
