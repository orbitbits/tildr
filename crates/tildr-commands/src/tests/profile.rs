use crate::profile::*;
use std::fs;
use std::path::PathBuf;
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
  let (root, ctx) = test_ctx("no-active");
  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::write(ctx.repo_path.join("profiles/default/.bashrc"), "default").unwrap();
  let profiles = Profiles::default();
  let result = profiles.resolve(&ctx.repo_path, ".bashrc");
  assert_eq!(result, ctx.repo_path.join("profiles/default/.bashrc"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn resolve_with_active_profile_uses_variant_on_disk() {
  let (root, ctx) = test_ctx("active-variant");
  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/archlinux")).unwrap();
  fs::write(ctx.repo_path.join("profiles/default/.bashrc"), "default").unwrap();
  fs::write(
    ctx.repo_path.join("profiles/archlinux/.bashrc"),
    "archlinux",
  )
  .unwrap();

  let profiles = Profiles {
    active: Some("archlinux".to_string()),
    ..Default::default()
  };
  let result = profiles.resolve(&ctx.repo_path, ".bashrc");
  assert_eq!(result, ctx.repo_path.join("profiles/archlinux/.bashrc"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn resolve_without_matching_file_uses_default() {
  let (root, ctx) = test_ctx("no-match");
  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/work")).unwrap();
  fs::write(ctx.repo_path.join("profiles/default/.bashrc"), "default").unwrap();
  fs::write(ctx.repo_path.join("profiles/default/.zshrc"), "default").unwrap();
  // .zshrc only exists in default, not in work

  let profiles = Profiles {
    active: Some("work".to_string()),
    ..Default::default()
  };
  let result = profiles.resolve(&ctx.repo_path, ".zshrc");
  assert_eq!(result, ctx.repo_path.join("profiles/default/.zshrc"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn profiles_save_and_load_roundtrip() {
  let (root, ctx) = test_ctx("roundtrip");

  let mut profiles = Profiles {
    active: Some("personal".to_string()),
    ..Default::default()
  };
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
  };
  let json = serde_json::to_string(&def).unwrap();
  assert!(json.contains("My profile"));
  assert!(!json.contains("files"));
}

#[test]
fn legacy_json_with_files_field_is_ignored() {
  let (root, ctx) = test_ctx("legacy");

  // Write a profiles.json with the old `files` field
  let legacy_json = r#"{
  "active": "archlinux",
  "profiles": {
    "default": {
      "description": null,
      "files": {
        ".bashrc": "profiles/default/.bashrc",
        ".zshrc": "profiles/default/.zshrc"
      }
    },
    "archlinux": {
      "description": "Arch Linux profile",
      "files": {
        ".bashrc": "profiles/archlinux/.bashrc"
      }
    }
  }
}"#;

  let tildr_dir = tildr_utils::fs::tildr_dir(&ctx.repo_path);
  fs::create_dir_all(&tildr_dir).unwrap();
  fs::write(tildr_dir.join("profiles.json"), legacy_json).unwrap();

  // Should load without error, ignoring the `files` fields
  let loaded = Profiles::load(&ctx).unwrap();
  assert_eq!(loaded.active, Some("archlinux".to_string()));
  assert!(loaded.profiles.contains_key("default"));
  assert!(loaded.profiles.contains_key("archlinux"));

  // The `files` field should not be present
  assert!(loaded.profiles["default"].description.is_none());
  assert_eq!(
    loaded.profiles["archlinux"].description.as_deref(),
    Some("Arch Linux profile")
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn drift_file_detected_by_resolve() {
  let (root, ctx) = test_ctx("drift");

  // Create profile directory with a file placed directly (no profile add)
  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/archlinux")).unwrap();
  fs::write(
    ctx.repo_path.join("profiles/default/.vimrc"),
    "default vimrc",
  )
  .unwrap();

  // Manually place .vimrc in archlinux profile WITHOUT using profile add
  // (simulates git pull or manual file placement)
  fs::write(
    ctx.repo_path.join("profiles/archlinux/.vimrc"),
    "archlinux vimrc",
  )
  .unwrap();

  // Activate archlinux profile
  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles.active = Some("archlinux".to_string());
  profiles.save(&ctx).unwrap();

  // resolve() should find the file in archlinux without any files map
  let loaded = Profiles::load(&ctx).unwrap();
  let resolved = loaded.resolve(&ctx.repo_path, ".vimrc");
  assert_eq!(
    resolved,
    ctx.repo_path.join("profiles/archlinux/.vimrc"),
    "resolve() should detect the manually-placed file in archlinux profile"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn variants_of_returns_correct_profiles() {
  let (root, ctx) = test_ctx("variants");

  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/archlinux")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/fedora")).unwrap();

  // .bashrc exists in default and archlinux, not in fedora
  fs::write(ctx.repo_path.join("profiles/default/.bashrc"), "d").unwrap();
  fs::write(ctx.repo_path.join("profiles/archlinux/.bashrc"), "a").unwrap();

  // .zshrc exists only in fedora
  fs::write(ctx.repo_path.join("profiles/fedora/.zshrc"), "f").unwrap();

  let known = vec![
    "default".to_string(),
    "archlinux".to_string(),
    "fedora".to_string(),
  ];

  let v1 = variants_of(&ctx.repo_path, ".bashrc", &known);
  assert_eq!(v1, vec!["archlinux", "default"]);

  let v2 = variants_of(&ctx.repo_path, ".zshrc", &known);
  assert_eq!(v2, vec!["fedora"]);

  let v3 = variants_of(&ctx.repo_path, ".nonexistent", &known);
  assert!(v3.is_empty());

  fs::remove_dir_all(&root).ok();
}
