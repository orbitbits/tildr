use crate::profile::*;
use std::fs;
use std::path::PathBuf;
use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_domain::ProfileMode;
use tildr_fs::symlink::is_symlink_to;

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
fn display_profile_name_labels_common_as_no_profile() {
  assert_eq!(display_profile_name(COMMON_PROFILE), "no profile");
  assert_eq!(display_profile_name("linux"), "linux");
}

#[test]
fn normalize_profile_name_accepts_no_profile_aliases() {
  assert_eq!(normalize_profile_name("no-profile"), COMMON_PROFILE);
  assert_eq!(normalize_profile_name("no_profile"), COMMON_PROFILE);
  assert_eq!(normalize_profile_name("no profile"), COMMON_PROFILE);
  assert_eq!(normalize_profile_name("linux"), "linux");
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
fn resolve_without_matching_default_uses_common() {
  let (root, ctx) = test_ctx("common-fallback");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.gitconfig"), "common").unwrap();

  let profiles = Profiles {
    active: Some("work".to_string()),
    ..Default::default()
  };
  let result = profiles.resolve(&ctx.repo_path, ".gitconfig");
  assert_eq!(result, ctx.repo_path.join("common/.gitconfig"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn resolve_supports_legacy_profiles_common() {
  let (root, ctx) = test_ctx("legacy-common-fallback");
  fs::create_dir_all(ctx.repo_path.join("profiles/common")).unwrap();
  fs::write(
    ctx.repo_path.join("profiles/common/.gitconfig"),
    "legacy common",
  )
  .unwrap();

  let profiles = Profiles::default();
  let result = profiles.resolve(&ctx.repo_path, ".gitconfig");
  assert_eq!(result, ctx.repo_path.join("profiles/common/.gitconfig"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn resolve_prefers_active_profile_over_common() {
  let (root, ctx) = test_ctx("active-over-common");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();

  let profiles = Profiles {
    active: Some("linux".to_string()),
    ..Default::default()
  };
  let result = profiles.resolve(&ctx.repo_path, ".bashrc");
  assert_eq!(result, ctx.repo_path.join("profiles/linux/.bashrc"));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn resolve_prefers_common_over_default() {
  let (root, ctx) = test_ctx("common-over-default");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/default")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/default/.bashrc"), "default").unwrap();

  let profiles = Profiles::default();
  let result = profiles.resolve(&ctx.repo_path, ".bashrc");
  assert_eq!(result, ctx.repo_path.join("common/.bashrc"));
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
fn profile_set_relinks_home_to_active_variant() {
  let (root, mut ctx) = test_ctx("set-relinks");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Set {
      name: "linux".to_string(),
    },
  )
  .unwrap();

  assert!(is_symlink_to(
    &ctx.home_path.join(".bashrc"),
    &ctx.repo_path.join("profiles/linux/.bashrc")
  ));

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_unset_relinks_home_to_common_variant() {
  let (root, mut ctx) = test_ctx("unset-relinks");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles.active = Some("linux".to_string());
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  crate::apply::run(
    &ctx,
    crate::apply::ApplyArgs {
      check: false,
      dry_run: false,
      force: false,
      verbose: false,
      quiet: true,
    },
  )
  .unwrap();
  assert!(is_symlink_to(
    &ctx.home_path.join(".bashrc"),
    &ctx.repo_path.join("profiles/linux/.bashrc")
  ));

  run(&ctx, &ProfileMode::Unset).unwrap();

  assert!(is_symlink_to(
    &ctx.home_path.join(".bashrc"),
    &ctx.repo_path.join("common/.bashrc")
  ));

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_migrate_moves_root_dotfiles_to_common() {
  let (root, mut ctx) = test_ctx("migrate-dotfiles");
  ctx.config.git.auto_commit = false;

  fs::write(ctx.repo_path.join(".bashrc"), "root bashrc").unwrap();
  fs::create_dir_all(ctx.repo_path.join(".tildr")).unwrap();
  fs::write(ctx.repo_path.join(".tildr/profiles.json"), "{}").unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles")).unwrap();

  run(&ctx, &ProfileMode::Migrate { dry_run: false }).unwrap();

  assert!(!ctx.repo_path.join(".bashrc").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("common/.bashrc")).unwrap(),
    "root bashrc"
  );
  assert!(ctx.repo_path.join(".tildr/profiles.json").exists());
  assert!(ctx.repo_path.join("profiles").is_dir());

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_migrate_keeps_repo_control_files_at_root() {
  let (root, mut ctx) = test_ctx("migrate-keeps-control-files");
  ctx.config.git.auto_commit = false;

  fs::write(ctx.repo_path.join(".bashrc"), "root bashrc").unwrap();
  fs::write(ctx.repo_path.join(".gitignore"), "target/").unwrap();
  fs::write(ctx.repo_path.join(".tildrignore"), "*.tmp").unwrap();

  run(&ctx, &ProfileMode::Migrate { dry_run: false }).unwrap();

  assert!(ctx.repo_path.join("common/.bashrc").exists());
  assert!(ctx.repo_path.join(".gitignore").exists());
  assert!(ctx.repo_path.join(".tildrignore").exists());
  assert!(!ctx.repo_path.join("common/.gitignore").exists());
  assert!(!ctx.repo_path.join("common/.tildrignore").exists());

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_migrate_dry_run_does_not_create_common() {
  let (root, mut ctx) = test_ctx("migrate-dry-run-no-side-effects");
  ctx.config.git.auto_commit = false;

  fs::write(ctx.repo_path.join(".bashrc"), "root bashrc").unwrap();

  run(&ctx, &ProfileMode::Migrate { dry_run: true }).unwrap();

  assert!(ctx.repo_path.join(".bashrc").exists());
  assert!(!ctx.repo_path.join("common").exists());

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_migrate_moves_legacy_profiles_common_to_common() {
  let (root, mut ctx) = test_ctx("migrate-legacy-common");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("profiles/common")).unwrap();
  fs::write(ctx.repo_path.join("profiles/common/.bashrc"), "legacy").unwrap();

  run(&ctx, &ProfileMode::Migrate { dry_run: false }).unwrap();

  assert!(!ctx.repo_path.join("profiles/common/.bashrc").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("common/.bashrc")).unwrap(),
    "legacy"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_delete_restores_orphans_to_common() {
  let (root, mut ctx) = test_ctx("delete-restores-common");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("profiles/work")).unwrap();
  fs::write(ctx.repo_path.join("profiles/work/.bashrc"), "work").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("work".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Delete {
      name: "work".to_string(),
    },
  )
  .unwrap();

  assert!(!ctx.repo_path.join("profiles/work").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("common/.bashrc")).unwrap(),
    "work"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_mv_accepts_no_profile_alias() {
  let (root, mut ctx) = test_ctx("mv-no-profile");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Mv {
      files: vec![".bashrc".to_string()],
      from: "no-profile".to_string(),
      to: "linux".to_string(),
    },
  )
  .unwrap();

  assert!(!ctx.repo_path.join("common/.bashrc").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("profiles/linux/.bashrc")).unwrap(),
    "common"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_mv_accepts_tilde_file_path() {
  let (root, mut ctx) = test_ctx("mv-tilde-file");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.xinitrc"), "xinit").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Mv {
      files: vec!["~/.xinitrc".to_string()],
      from: "no-profile".to_string(),
      to: "linux".to_string(),
    },
  )
  .unwrap();

  assert!(!ctx.repo_path.join("common/.xinitrc").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("profiles/linux/.xinitrc")).unwrap(),
    "xinit"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_mv_accepts_home_env_file_path() {
  let (root, mut ctx) = test_ctx("mv-home-env-file");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.xprofile"), "xprofile").unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Mv {
      files: vec!["$HOME/.xprofile".to_string()],
      from: "no-profile".to_string(),
      to: "linux".to_string(),
    },
  )
  .unwrap();

  assert!(!ctx.repo_path.join("common/.xprofile").exists());
  assert_eq!(
    fs::read_to_string(ctx.repo_path.join("profiles/linux/.xprofile")).unwrap(),
    "xprofile"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_mv_removes_empty_source_directories() {
  let (root, mut ctx) = test_ctx("mv-cleans-empty-dirs");
  ctx.config.git.auto_commit = false;

  fs::create_dir_all(ctx.repo_path.join("common/.config/program")).unwrap();
  fs::write(
    ctx.repo_path.join("common/.config/program/config.conf"),
    "config",
  )
  .unwrap();

  let mut profiles = Profiles::load(&ctx).unwrap();
  profiles
    .profiles
    .insert("linux".to_string(), ProfileDef::default());
  profiles.save(&ctx).unwrap();

  run(
    &ctx,
    &ProfileMode::Mv {
      files: Vec::new(),
      from: "no-profile".to_string(),
      to: "linux".to_string(),
    },
  )
  .unwrap();

  assert!(!ctx.repo_path.join("common/.config/program").exists());
  assert!(!ctx.repo_path.join("common/.config").exists());
  assert!(ctx.repo_path.join("common").exists());
  assert_eq!(
    fs::read_to_string(
      ctx
        .repo_path
        .join("profiles/linux/.config/program/config.conf")
    )
    .unwrap(),
    "config"
  );

  fs::remove_dir_all(&root).ok();
}

#[test]
fn clean_removes_empty_profile_storage_directories() {
  let (root, ctx) = test_ctx("clean-empty-dirs");

  fs::create_dir_all(ctx.repo_path.join("common/.config/empty")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux/.cache/empty")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux/.config")).unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.config/keep"), "keep").unwrap();

  let removed = crate::clean::clean_empty_profile_dirs(&ctx, false).unwrap();

  assert!(removed.contains(&PathBuf::from("common/.config/empty")));
  assert!(removed.contains(&PathBuf::from("common/.config")));
  assert!(removed.contains(&PathBuf::from("profiles/linux/.cache/empty")));
  assert!(removed.contains(&PathBuf::from("profiles/linux/.cache")));
  assert!(!ctx.repo_path.join("common/.config/empty").exists());
  assert!(!ctx.repo_path.join("profiles/linux/.cache").exists());
  assert!(ctx.repo_path.join("profiles/linux/.config/keep").exists());
  assert!(ctx.repo_path.join("common").exists());
  assert!(ctx.repo_path.join("profiles/linux").exists());

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
