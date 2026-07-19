use crate::utils::target::{FileResolution, ResolvedTarget, resolve_logical_file, resolve_target};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tildr_core::config::Config;
use tildr_core::context::Context;

fn cwd_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

fn test_ctx(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tildr-test-target-{name}-{nanos}"));
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

fn create_profile_file(ctx: &Context, profile: &str, file: &str, content: &str) {
  let dir = crate::profile::profile_dir(&ctx.repo_path, profile);
  fs::create_dir_all(&dir).unwrap();
  let path = dir.join(file);
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).unwrap();
  }
  fs::write(path, content).unwrap();
}

fn set_active(ctx: &Context, name: &str) {
  let mut profiles = crate::profile::Profiles::load(ctx).unwrap();
  profiles.active = Some(name.to_string());
  profiles.save(ctx).unwrap();
}

#[test]
fn file_only_in_active_profile_resolves() {
  let (root, ctx) = test_ctx("active-only");
  create_profile_file(&ctx, "archlinux", ".bashrc", "arch");
  set_active(&ctx, "archlinux");

  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::Found(entry) => {
      assert_eq!(entry.profile, "archlinux");
      assert_eq!(entry.repo_path.file_name().unwrap(), ".bashrc");
    }
    _ => panic!("Expected Found in archlinux"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_only_in_default_no_active_resolves_to_default() {
  let (root, ctx) = test_ctx("default-only");
  create_profile_file(&ctx, "default", ".bashrc", "default");

  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::Found(entry) => {
      assert_eq!(entry.profile, "default");
    }
    _ => panic!("Expected Found in default"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_in_active_and_common_resolves_to_active() {
  let (root, ctx) = test_ctx("active-common");
  create_profile_file(&ctx, "common", ".bashrc", "common");
  create_profile_file(&ctx, "linux", ".bashrc", "linux");
  set_active(&ctx, "linux");

  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::Found(entry) => {
      assert_eq!(entry.profile, "linux");
      assert_eq!(
        entry.repo_path,
        ctx.repo_path.join("profiles/linux/.bashrc")
      );
    }
    _ => panic!("Expected Found in linux"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_only_in_common_no_active_resolves_to_common() {
  let (root, ctx) = test_ctx("common-no-active");
  create_profile_file(&ctx, "common", ".bashrc", "common");

  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::Found(entry) => {
      assert_eq!(entry.profile, "common");
    }
    _ => panic!("Expected Found in common"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_in_active_and_other_resolves_to_active_deterministically() {
  let (root, ctx) = test_ctx("deterministic");
  create_profile_file(&ctx, "archlinux", ".bashrc", "arch");
  create_profile_file(&ctx, "fedora", ".bashrc", "fedora");
  create_profile_file(&ctx, "default", ".bashrc", "default");
  set_active(&ctx, "archlinux");

  // Run 20 times to ensure deterministic behavior despite parallel scan
  for i in 0..20 {
    let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
    match result {
      FileResolution::Found(entry) => {
        assert_eq!(
          entry.profile, "archlinux",
          "Run {i}: expected archlinux, got {}",
          entry.profile
        );
      }
      _ => panic!("Run {i}: Expected Found in archlinux"),
    }
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_only_in_other_profile_returns_ambiguous() {
  let (root, ctx) = test_ctx("ambiguous");
  create_profile_file(&ctx, "fedora", ".bashrc", "fedora");
  create_profile_file(&ctx, "default", ".zshrc", "default");
  set_active(&ctx, "archlinux");

  // .bashrc exists only in fedora, not in archlinux (active) nor default
  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::AmbiguousAcrossProfiles(profiles) => {
      assert!(profiles.contains(&"fedora".to_string()));
      assert!(!profiles.contains(&"archlinux".to_string()));
    }
    _ => panic!("Expected AmbiguousAcrossProfiles"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn file_only_in_other_profile_no_active_returns_ambiguous() {
  let (root, ctx) = test_ctx("ambiguous-no-active");
  create_profile_file(&ctx, "fedora", ".bashrc", "fedora");
  // No active profile, .bashrc only in fedora (not default)

  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), None).unwrap();
  match result {
    FileResolution::AmbiguousAcrossProfiles(profiles) => {
      assert!(profiles.contains(&"fedora".to_string()));
    }
    _ => panic!("Expected AmbiguousAcrossProfiles"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn ambiguous_file_resolves_with_profile_override() {
  let (root, ctx) = test_ctx("override");
  create_profile_file(&ctx, "fedora", ".bashrc", "fedora");
  set_active(&ctx, "archlinux");

  // With --profile fedora, should resolve directly
  let result = resolve_logical_file(&ctx, Path::new(".bashrc"), Some("fedora")).unwrap();
  match result {
    FileResolution::Found(entry) => {
      assert_eq!(entry.profile, "fedora");
    }
    _ => panic!("Expected Found in fedora with --profile"),
  }
  fs::remove_dir_all(&root).ok();
}

#[test]
fn not_managed_file_returns_not_managed() {
  let (root, ctx) = test_ctx("not-managed");
  create_profile_file(&ctx, "default", ".bashrc", "default");

  let result = resolve_logical_file(&ctx, Path::new(".nonexistent"), None).unwrap();
  assert!(matches!(result, FileResolution::NotManaged));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn common_storage_target_resolves_to_home_relative_file() {
  let (root, ctx) = test_ctx("common-storage-target");
  create_profile_file(&ctx, "common", ".wgetrc", "common");

  let result = resolve_target(&ctx, Some("common/.wgetrc".to_string()), None).unwrap();
  match result {
    ResolvedTarget::File(entry) => {
      assert_eq!(entry.profile, "common");
      assert_eq!(entry.relative, PathBuf::from(".wgetrc"));
      assert_eq!(entry.repo_path, ctx.repo_path.join("common/.wgetrc"));
    }
    _ => panic!("Expected common/.wgetrc to resolve as a managed file"),
  }

  fs::remove_dir_all(&root).ok();
}

#[test]
fn profile_storage_target_resolves_to_home_relative_file() {
  let (root, ctx) = test_ctx("profile-storage-target");
  create_profile_file(&ctx, "linux", ".bashrc", "linux");

  let result = resolve_target(&ctx, Some("profiles/linux/.bashrc".to_string()), None).unwrap();
  match result {
    ResolvedTarget::File(entry) => {
      assert_eq!(entry.profile, "linux");
      assert_eq!(entry.relative, PathBuf::from(".bashrc"));
      assert_eq!(
        entry.repo_path,
        ctx.repo_path.join("profiles/linux/.bashrc")
      );
    }
    _ => panic!("Expected profiles/linux/.bashrc to resolve as a managed file"),
  }

  fs::remove_dir_all(&root).ok();
}

#[test]
fn home_env_target_resolves_to_home_relative_file() {
  let (root, ctx) = test_ctx("home-env-target");
  create_profile_file(&ctx, "common", ".config/starship.toml", "starship");

  let target = "$HOME/.config/starship.toml".to_string();
  let result = resolve_target(&ctx, Some(target), None).unwrap();
  match result {
    ResolvedTarget::File(entry) => {
      assert_eq!(entry.profile, "common");
      assert_eq!(entry.relative, PathBuf::from(".config/starship.toml"));
    }
    _ => panic!("Expected $HOME/.config/starship.toml to resolve as a managed file"),
  }

  fs::remove_dir_all(&root).ok();
}

#[test]
fn cwd_relative_target_under_home_resolves_to_home_relative_file() {
  let _guard = cwd_lock().lock().unwrap();
  let old_cwd = std::env::current_dir().unwrap();
  let (root, ctx) = test_ctx("cwd-relative-target");
  let documents = ctx.home_path.join("Documents");
  fs::create_dir_all(&documents).unwrap();
  create_profile_file(&ctx, "common", "Documents/document.ods", "document");

  std::env::set_current_dir(&documents).unwrap();
  let result = resolve_target(&ctx, Some("document.ods".to_string()), None).unwrap();
  match result {
    ResolvedTarget::File(entry) => {
      assert_eq!(entry.profile, "common");
      assert_eq!(entry.relative, PathBuf::from("Documents/document.ods"));
    }
    _ => panic!("Expected document.ods to resolve relative to cwd under HOME"),
  }

  std::env::set_current_dir(old_cwd).unwrap();
  fs::remove_dir_all(&root).ok();
}
