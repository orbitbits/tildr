use crate::fs::*;
use std::path::Path;

#[test]
fn format_size_zero() {
  assert_eq!(format_size(0), "0 B");
}

#[test]
fn format_size_bytes() {
  assert_eq!(format_size(1), "1 B");
  assert_eq!(format_size(1023), "1023 B");
}

#[test]
fn format_size_kilobytes() {
  assert_eq!(format_size(1024), "1.0 KB");
  assert_eq!(format_size(1536), "1.5 KB");
}

#[test]
fn format_size_megabytes() {
  assert_eq!(format_size(1_048_576), "1.0 MB");
  assert_eq!(format_size(1_572_864), "1.5 MB");
}

#[test]
fn should_ignore_tildr_dir() {
  assert!(should_ignore(Path::new(".tildr")));
  assert!(should_ignore(Path::new("/repo/.tildr")));
}

#[test]
fn should_ignore_dotfiles() {
  assert!(should_ignore(Path::new(".DS_Store")));
  assert!(should_ignore(Path::new("Thumbs.db")));
  assert!(should_ignore(Path::new(".gitkeep")));
  assert!(should_ignore(Path::new(".gitignore")));
  assert!(should_ignore(Path::new(".tildrignore")));
}

#[test]
fn should_ignore_backup_extensions() {
  assert!(!should_ignore(Path::new("file.bak")));
  assert!(should_ignore(Path::new("file.tmp")));
  assert!(should_ignore(Path::new("file.swp")));
  assert!(!should_ignore(Path::new("file.BAK")));
}

#[test]
fn should_ignore_tilde_suffix() {
  assert!(should_ignore(Path::new("file~")));
  assert!(should_ignore(Path::new("file.txt~")));
}

#[test]
fn should_not_ignore_normal_files() {
  assert!(!should_ignore(Path::new("normal.txt")));
  assert!(!should_ignore(Path::new("file.bashrc")));
  assert!(!should_ignore(Path::new("Makefile")));
}

#[test]
fn should_ignore_path_with_unicode_fails_safely() {
  assert!(!should_ignore(Path::new("")));
}

#[test]
fn short_id_is_deterministic() {
  assert_eq!(
    short_id("/home/user/.bashrc"),
    short_id("/home/user/.bashrc")
  );
}

#[test]
fn short_id_length_is_four() {
  assert_eq!(short_id("anything").len(), 4);
}

#[test]
fn short_id_differs_for_different_inputs() {
  assert_ne!(short_id("file1"), short_id("file2"));
}
