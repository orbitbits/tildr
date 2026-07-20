use crate::list::{ListArgs, run};
use std::fs;
use std::path::PathBuf;
use tildr_core::config::Config;
use tildr_core::context::Context;
use tildr_fs::symlink::is_symlink_to;

fn test_context(name: &str) -> (PathBuf, Context) {
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tildr-test-list-{name}-{nanos}"));
  let home = root.join("home");
  let repo = root.join("repo");
  fs::create_dir_all(&home).unwrap();
  fs::create_dir_all(&repo).unwrap();
  let mut config = Config::default();
  config.core.repo = repo.to_string_lossy().to_string();
  (
    root,
    Context {
      config,
      repo_path: repo,
      home_path: home,
    },
  )
}

fn list_args() -> ListArgs {
  ListArgs {
    tree: false,
    long: false,
    source: false,
    export: None,
    import: None,
    less: false,
    profile: None,
  }
}

fn set_active(ctx: &Context, name: &str) {
  let mut profiles = crate::profile::Profiles::load(ctx).unwrap();
  profiles.active = Some(name.to_string());
  profiles.save(ctx).unwrap();
}

#[test]
fn list_export_contains_effective_home_relative_paths() {
  let (root, ctx) = test_context("export-logical");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/work")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();
  fs::write(ctx.repo_path.join("profiles/work/.workrc"), "work").unwrap();
  set_active(&ctx, "linux");
  let output = root.join("files.json");
  let mut args = list_args();
  args.export = Some(output.display().to_string());

  run(&ctx, args).unwrap();

  let json: serde_json::Value =
    serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
  assert_eq!(json["files"], serde_json::json!([".bashrc"]));
  fs::remove_dir_all(&root).ok();
}

#[test]
fn list_import_maps_legacy_storage_paths_to_effective_home_links() {
  let (root, ctx) = test_context("import-storage");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::create_dir_all(ctx.repo_path.join("profiles/linux")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "common").unwrap();
  fs::write(ctx.repo_path.join("profiles/linux/.bashrc"), "linux").unwrap();
  set_active(&ctx, "linux");
  let input = root.join("files.json");
  fs::write(
    &input,
    r#"{"version":1,"files":["common/.bashrc","profiles/linux/.bashrc"]}"#,
  )
  .unwrap();
  let mut args = list_args();
  args.import = Some(input.display().to_string());

  run(&ctx, args).unwrap();

  assert!(is_symlink_to(
    &ctx.home_path.join(".bashrc"),
    &ctx.repo_path.join("profiles/linux/.bashrc")
  ));
  assert!(!ctx.home_path.join("common/.bashrc").exists());
  assert!(!ctx.home_path.join("profiles/linux/.bashrc").exists());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn list_import_rejects_paths_outside_home() {
  let (root, ctx) = test_context("import-escape");
  let input = root.join("files.json");
  fs::write(&input, r#"{"version":1,"files":["../../etc/passwd"]}"#).unwrap();
  let mut args = list_args();
  args.import = Some(input.display().to_string());

  let result = run(&ctx, args);

  assert!(result.is_err());
  fs::remove_dir_all(&root).ok();
}

#[test]
fn list_import_preserves_regular_home_conflicts() {
  let (root, ctx) = test_context("import-conflict");
  fs::create_dir_all(ctx.repo_path.join("common")).unwrap();
  fs::write(ctx.repo_path.join("common/.bashrc"), "repo").unwrap();
  fs::write(ctx.home_path.join(".bashrc"), "home").unwrap();
  let input = root.join("files.json");
  fs::write(&input, r#"{"version":1,"files":[".bashrc"]}"#).unwrap();
  let mut args = list_args();
  args.import = Some(input.display().to_string());

  run(&ctx, args).unwrap();

  assert_eq!(
    fs::read_to_string(ctx.home_path.join(".bashrc")).unwrap(),
    "home"
  );
  assert!(!ctx.home_path.join(".bashrc").is_symlink());
  fs::remove_dir_all(&root).ok();
}
