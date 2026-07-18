use crate::profile::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tildr_core::config::Config;
use tildr_core::context::Context;

fn test_ctx(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tildr-test-profile-{name}-{nanos}"));
  let repo = root.join("repo");
  let home = root.join("home");
  fs::create_dir_all(&repo).unwrap();
  fs::create_dir_all(&home).unwrap();
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  let ctx = Context {
    config,
    repo_path: repo,
    home_path: home,
  };
  (root, ctx)
}

#[test]
fn resolve_without_active_profile_uses_default() {
  let profiles = Profiles::default();
  let result = profiles.resolve(Path::new("/repo"), ".bashrc");
  assert_eq!(result, PathBuf::from("/repo/.bashrc"));
}

#[test]
fn resolve_with_active_profile_uses_variant() {
  let mut profiles = Profiles::default();
  profiles.active = Some("work".to_string());
  profiles.profiles.insert(
    "work".to_string(),
    ProfileDef {
      description: None,
      files: [(".bashrc".to_string(), "profiles/work/.bashrc".to_string())]
        .into_iter()
        .collect(),
    },
  );
  let result = profiles.resolve(Path::new("/repo"), ".bashrc");
  assert_eq!(result, PathBuf::from("/repo/profiles/work/.bashrc"));
}

#[test]
fn resolve_without_matching_file_uses_default() {
  let mut profiles = Profiles::default();
  profiles.active = Some("work".to_string());
  profiles.profiles.insert(
    "work".to_string(),
    ProfileDef {
      description: None,
      files: HashMap::new(),
    },
  );
  let result = profiles.resolve(Path::new("/repo"), ".zshrc");
  assert_eq!(result, PathBuf::from("/repo/.zshrc"));
}

#[test]
fn profiles_save_and_load_roundtrip() {
  let (root, ctx) = test_ctx("roundtrip");

  let mut profiles = Profiles::default();
  profiles.active = Some("personal".to_string());
  profiles
    .profiles
    .insert("personal".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  let loaded = Profiles::load(&ctx).unwrap();
  assert_eq!(loaded.active, Some("personal".to_string()));
  assert!(loaded.profiles.contains_key("personal"));

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_create_adds_new_profile() {
  let (root, ctx) = test_ctx("create");

  Profiles::load(&ctx).unwrap().save(&ctx).unwrap();

  let loaded = Profiles::load(&ctx).unwrap();
  assert!(loaded.active.is_none());

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profiles_serialization() {
  let profiles = Profiles::default();
  let json = serde_json::to_string(&profiles).unwrap();
  assert_eq!(json, r#"{"active":null,"profiles":{}}"#);
}

#[test]
fn profile_def_serialization() {
  let def = ProfileDef {
    description: Some("My profile".to_string()),
    files: [(".bashrc".to_string(), "profiles/work/.bashrc".to_string())]
      .into_iter()
      .collect(),
  };
  let json = serde_json::to_string(&def).unwrap();
  assert!(json.contains("My profile"));
  assert!(json.contains(".bashrc"));
}
