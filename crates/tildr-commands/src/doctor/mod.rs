mod checks;
mod output;
mod utils;

use anyhow::Result;
use std::sync::OnceLock;
use tildr_core::context::Context;
use tildr_repo::ManagedEntry;
use tildr_ui::{color::Colorize, icons, info, success};

use self::checks::*;
use crate::utils::target::scan_effective_entries;

pub fn run(ctx: &Context) -> Result<()> {
  Doctor::new(ctx).run()
}

struct Doctor<'a> {
  ctx: &'a Context,
  entries: OnceLock<Vec<ManagedEntry>>,
  issues: usize,
  failed_checks: usize,
}

impl<'a> Doctor<'a> {
  fn new(ctx: &'a Context) -> Self {
    Self {
      ctx,
      entries: OnceLock::new(),
      issues: 0,
      failed_checks: 0,
    }
  }

  fn run(mut self) -> Result<()> {
    println!("Checking environment...\n");

    let checks: [&dyn DoctorCheck; 6] = [
      &RepositoryCheck,
      &ConfigCheck,
      &GitCheck,
      &PermissionsCheck,
      &DiskCheck,
      &SymlinkCheck,
    ];

    for check in checks {
      let report = check.run(&self)?;
      self.issues += report.issues;
      if report.failed() {
        self.failed_checks += 1;
      }
      report.print();
    }

    println!();

    if self.issues == 0 {
      success("All checks passed");
    } else {
      info(&format!(
        "{} check{} failed, {} issue{} found",
        self.failed_checks,
        if self.failed_checks == 1 { "" } else { "s" },
        self.issues,
        if self.issues == 1 { "" } else { "s" }
      ));
    }

    Ok(())
  }

  fn repo_exists(&self) -> bool {
    self.ctx.repo_path.exists()
  }

  fn repo_entries(&self) -> Result<Option<&[ManagedEntry]>> {
    if !self.repo_exists() {
      return Ok(None);
    }

    if self.entries.get().is_none() {
      let entries = scan_effective_entries(self.ctx)?;
      let _ = self.entries.set(entries);
    }

    Ok(self.entries.get().map(Vec::as_slice))
  }
}

trait DoctorCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport>;
}

#[derive(Clone, Copy)]
enum CheckStatus {
  Ok,
  Fail,
}

struct CheckReport {
  name: &'static str,
  status: CheckStatus,
  issues: usize,
  extra: Option<String>,
  hint: Option<String>,
}

impl CheckReport {
  fn ok(name: &'static str) -> Self {
    Self {
      name,
      status: CheckStatus::Ok,
      issues: 0,
      extra: None,
      hint: None,
    }
  }

  fn fail(name: &'static str, issues: usize) -> Self {
    Self {
      name,
      status: CheckStatus::Fail,
      issues,
      extra: None,
      hint: None,
    }
  }

  fn from_issues(name: &'static str, issues: usize) -> Self {
    if issues == 0 {
      Self::ok(name)
    } else {
      Self::fail(name, issues)
    }
  }

  fn with_extra(mut self, extra: impl Into<String>) -> Self {
    self.extra = Some(extra.into());
    self
  }

  fn with_hint(mut self, hint: impl Into<String>) -> Self {
    self.hint = Some(hint.into());
    self
  }

  fn failed(&self) -> bool {
    matches!(self.status, CheckStatus::Fail)
  }

  fn print(&self) {
    let status = match self.status {
      CheckStatus::Ok => "OK".to_string().green(),
      CheckStatus::Fail => format!("{}FAIL", icons().cross).red(),
    };
    let symbol = match self.status {
      CheckStatus::Ok => icons().none,
      CheckStatus::Fail => icons().none,
    };

    match (&self.extra, &self.hint) {
      (Some(extra), Some(hint)) => {
        println!("{symbol}{:<12} {} ({extra}) → {hint}", self.name, status)
      }
      (Some(extra), None) => println!("{symbol}{:<12} {} ({extra})", self.name, status),
      (None, Some(hint)) => println!("{symbol}{:<12} {} → {hint}", self.name, status),
      (None, None) => println!("{symbol}{:<12} {}", self.name, status),
    }
  }
}
