use anyhow::Result;
use std::fs::File;
use tildr_core::config::Config;
use tildr_fs::symlink::{is_symlink, is_symlink_to};
use tildr_git::{GitIntegration, GitStatusIssueKind};
use tildr_utils::fs::format_size;

use super::output::compact_issue_summary;
use super::utils::{check_repo_permissions, repo_size};
use super::{CheckReport, Doctor, DoctorCheck};
use crate::profile::Profiles;

pub(super) struct RepositoryCheck;

impl DoctorCheck for RepositoryCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport> {
    Ok(if doctor.repo_exists() {
      CheckReport::ok("Repository")
    } else {
      CheckReport::fail("Repository", 1)
    })
  }
}

pub(super) struct ConfigCheck;

impl DoctorCheck for ConfigCheck {
  fn run(&self, _doctor: &Doctor<'_>) -> Result<CheckReport> {
    Ok(if Config::config_path().exists() {
      CheckReport::ok("Config")
    } else {
      CheckReport::fail("Config", 1)
    })
  }
}

pub(super) struct GitCheck;

impl DoctorCheck for GitCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport> {
    let git = GitIntegration::new(doctor.ctx.repo_path.clone());

    if !git.is_git_repo() {
      return Ok(CheckReport::fail("Git", 1));
    }

    if !doctor.ctx.config.git.available {
      return Ok(
        CheckReport::fail("Git", 1)
          .with_extra("git unavailable")
          .with_hint("tildr init"),
      );
    }

    let issues = match git.status_issues() {
      Ok(issues) => issues,
      Err(err) => {
        return Ok(
          CheckReport::fail("Git", 1)
            .with_extra(format!("status check failed: {err}"))
            .with_hint("git status"),
        );
      }
    };

    if issues.is_empty() {
      return Ok(CheckReport::ok("Git"));
    }

    let pending_changes = issues
      .iter()
      .filter(|issue| {
        matches!(
          issue.kind,
          GitStatusIssueKind::Untracked | GitStatusIssueKind::Uncommitted
        )
      })
      .count();

    Ok(
      CheckReport::fail("Git", issues.len())
        .with_extra(format!(
          "{} pending change{}",
          pending_changes,
          if pending_changes == 1 { "" } else { "s" }
        ))
        .with_hint("tildr git status"),
    )
  }
}

pub(super) struct PermissionsCheck;

impl DoctorCheck for PermissionsCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport> {
    let Some(entries) = doctor.repo_entries()? else {
      return Ok(CheckReport::ok("Permissions"));
    };

    let mut issues = 0usize;
    let mut denied_files = 0usize;
    let mut repo_access_issue = false;

    if !check_repo_permissions(&doctor.ctx.repo_path) {
      issues += 1;
      repo_access_issue = true;
    }

    for entry in entries {
      let home_path = doctor.ctx.home_path.join(&entry.relative);

      if home_path.exists() && File::open(&home_path).is_err() {
        issues += 1;
        denied_files += 1;
      }
    }

    let mut report = CheckReport::from_issues("Permissions", issues);

    if issues > 0 {
      let mut parts = Vec::new();
      if repo_access_issue {
        parts.push("1 Repository access issue".to_string());
      }
      if denied_files > 0 {
        parts.push(format!("{denied_files} Permission denied"));
      }
      report = report.with_extra(parts.join(", "));
    }

    Ok(report)
  }
}

pub(super) struct DiskCheck;

impl DoctorCheck for DiskCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport> {
    let size = repo_size(&doctor.ctx.repo_path);
    Ok(CheckReport::ok("Disk").with_extra(format_size(size)))
  }
}

pub(super) struct SymlinkCheck;

impl DoctorCheck for SymlinkCheck {
  fn run(&self, doctor: &Doctor<'_>) -> Result<CheckReport> {
    let Some(entries) = doctor.repo_entries()? else {
      return Ok(CheckReport::ok("Symlinks"));
    };

    let profiles = Profiles::load(doctor.ctx)?;
    let mut issues = 0usize;
    let mut broken_links = 0usize;
    let mut missing_links = 0usize;

    for entry in entries {
      let home_link = doctor.ctx.home_path.join(&entry.relative);
      let file_str = entry.relative.display().to_string();
      let expected = profiles.resolve(&doctor.ctx.repo_path, &file_str);

      if is_symlink(&home_link) {
        if !is_symlink_to(&home_link, &expected) {
          issues += 1;
          broken_links += 1;
        }
      } else if !home_link.exists() {
        issues += 1;
        missing_links += 1;
      }
    }

    let mut report = CheckReport::from_issues("Symlinks", issues);

    if issues > 0 {
      report = report
        .with_extra(compact_issue_summary(&[
          (broken_links, "Broken symlink"),
          (missing_links, "Missing link"),
        ]))
        .with_hint("tildr status");
    }

    Ok(report)
  }
}
