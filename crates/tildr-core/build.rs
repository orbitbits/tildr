// (c) 2026 OrbitBits. All rights reserved.
use std::{env::var, fs, path::Path, process::Command};

pub struct GitCommands;

pub fn last_update() -> String {
  Command::new("git")
    .args(["show", "-s", "--format=%cs", "HEAD"])
    .output()
    .ok()
    .and_then(|o| {
      o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
    })
    .unwrap_or_else(|| "unknown".into())
}

pub fn latest_commit() -> String {
  let hash = Command::new("git")
    .args(["rev-parse", "HEAD"])
    .output()
    .ok()
    .and_then(|output| {
      if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
      } else {
        None
      }
    })
    .unwrap_or_else(|| "unknown".to_string());

  // I only take the first 8 characters of the commit hash.
  hash[..8].to_string()
}

fn metadata(key: &str) -> Result<String, Box<dyn std::error::Error>> {
  // CARGO_MANIFEST_DIR points to where the build.rs file is located (the root of the workspace).
  let manifest_dir = var("CARGO_MANIFEST_DIR")?;

  let root = Path::new(&manifest_dir)
    .ancestors()
    .find(|p| {
      let cargo = p.join("Cargo.toml");
      if !cargo.exists() {
        return false;
      }

      if let Ok(content) = fs::read_to_string(&cargo) {
        content.contains("[workspace]")
      } else {
        false
      }
    })
    .expect("Workspace Cargo.toml not found");

  // Go up one level to get the Cargo.toml file from the workspace.
  let workspace_cargo_toml = root.join("Cargo.toml");

  let content = fs::read_to_string(&workspace_cargo_toml)?;
  let toml: toml::Value = toml::from_str(&content)?;
  let value = toml
    .get("workspace")
    .and_then(|w| w.get("metadata"))
    .and_then(|w| w.get("tildr"))
    .and_then(|m| m.get(key))
    .and_then(|c| c.as_str())
    .unwrap_or("Unknown");

  Ok(value.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!(
    "cargo:rustc-env=CARGO_PKG_LATEST_COMMIT={}",
    latest_commit()
  );

  println!("cargo:rustc-env=CARGO_PKG_LAST_UPDATE={}", last_update());

  println!(
    "cargo:rustc-env=CARGO_PKG_COPYRIGHT={}",
    metadata("copyright").unwrap_or_else(|_| "unknown".into())
  );

  println!(
    "cargo:rustc-env=CARGO_PKG_MAINTAINER={}",
    metadata("maintainer").unwrap_or_else(|_| "unknown".into())
  );

  println!(
    "cargo:rustc-env=CARGO_PKG_DOCUMENTATION={}",
    metadata("documentation").unwrap_or_else(|_| "unknown".into())
  );

  println!(
    "cargo:rustc-env=CARGO_PKG_KEYWORDS={}",
    metadata("keywords").unwrap_or_else(|_| "unknown".into())
  );

  println!(
    "cargo:rustc-env=CARGO_PKG_CATEGORIES={}",
    metadata("categories").unwrap_or_else(|_| "unknown".into())
  );

  let profile = var("PROFILE").unwrap_or_else(|_| "unknown".into());
  println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

  Ok(())
}
