use anyhow::{Result, bail};
use std::path::PathBuf;
use tildr_core::{build_info::BuildInfo, constants::APP_NAME, context::Context};
use tildr_domain::InfoMode;
use tildr_ui::color::Colorize;
use tildr_utils::{output::authors_format, pager::page_string, traits::Capitalize};

pub struct InfoArgs {
  pub mode: InfoMode,
}

pub fn run(_ctx: &Context, args: InfoArgs) -> Result<()> {
  match args.mode {
    InfoMode::License => run_license(),
    InfoMode::Credits => run_credits(),
  }
}

fn run_license() -> Result<()> {
  let candidates = license_candidates(APP_NAME);

  let path = candidates.iter().map(PathBuf::from).find(|p| p.exists());

  let path = match path {
    Some(p) => p,
    None => {
      bail!(
        "License file not found.\nSee: https://orbitbits.github.io/{}/license/",
        APP_NAME
      )
    }
  };

  let content = std::fs::read_to_string(&path)?;

  page_string(&content)?; // Always use pager here (better UX)

  Ok(())
}

fn license_candidates(app: &str) -> Vec<PathBuf> {
  let mut paths = Vec::new();

  #[cfg(target_os = "linux")]
  {
    paths.push(format!("/usr/share/licenses/{}/LICENSE", app).into());
    paths.push(format!("/usr/local/share/licenses/{}/LICENSE", app).into());
  }

  #[cfg(target_os = "macos")]
  {
    let prefix = if PathBuf::from("/opt/homebrew").exists() {
      "/opt/homebrew"
    } else {
      "/usr/local"
    };

    paths.push(format!("{}/share/doc/{}/LICENSE", prefix, app).into());
  }

  paths
}

fn run_credits() -> Result<()> {
  let sep: String = "-".repeat(55);
  let app_name = BuildInfo::name()
    .to_string()
    .capitalize()
    .replace("-core", "");

  let result = format!(
    r#"
{} - {} {}

{}:
  * {}:
  {}
  * {}: {}

{}: {}
{}: {}
{}: {}
{}: {}

{sep}
{}
{}: {}
{sep}"#,
    app_name.magenta().bold(),
    "Version".bold(),
    BuildInfo::version(),
    "Credits".cyan(),
    "Authors".bold(),
    authors_format(env!("CARGO_PKG_AUTHORS"), "  - "),
    "Maintainer".bold(),
    BuildInfo::maintainer(),
    "Repository".cyan(),
    BuildInfo::repository(),
    "License".cyan(),
    BuildInfo::license(),
    "Commit".cyan(),
    BuildInfo::latest_commit(),
    "Last update".cyan(),
    BuildInfo::last_update(),
    BuildInfo::copyright().bold(),
    "Homepage".cyan(),
    BuildInfo::homepage()
  );

  //   let logo = r#"
  //            ..
  //        ...........
  //     .................
  //   .....................
  //   ....             ....
  //   .........   .........
  //   .........   .........
  //   .........   .........
  //   .........   .........
  //      ......   ......
  //         ...   ...
  // "#
  //   .green();
  //   println!("{}", logo);
  println!("{}", result);
  Ok(())
}
