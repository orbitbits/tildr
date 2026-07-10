pub struct BuildInfo;

impl BuildInfo {
  pub fn name() -> &'static str {
    env!("CARGO_PKG_NAME")
  }

  pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
  }

  pub fn maintainer() -> &'static str {
    env!("CARGO_PKG_MAINTAINER")
  }

  pub fn repository() -> &'static str {
    env!("CARGO_PKG_REPOSITORY")
  }

  pub fn license() -> &'static str {
    env!("CARGO_PKG_LICENSE")
  }

  pub fn homepage() -> &'static str {
    env!("CARGO_PKG_HOMEPAGE")
  }

  pub fn copyright() -> &'static str {
    env!("CARGO_PKG_COPYRIGHT")
  }

  pub fn latest_commit() -> &'static str {
    env!("CARGO_PKG_LATEST_COMMIT")
  }

  pub fn last_update() -> &'static str {
    env!("CARGO_PKG_LAST_UPDATE")
  }
}
