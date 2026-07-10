#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncScenario {
  UpToDate,
  PushOnly {
    local_ahead: usize,
  },
  PullOnly {
    remote_ahead: usize,
  },
  Diverged {
    local_ahead: usize,
    remote_ahead: usize,
  },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeCheck {
  Clean,
  Conflicted(Vec<String>),
}

pub fn classify_sync_scenario(local_ahead: usize, remote_ahead: usize) -> SyncScenario {
  match (local_ahead, remote_ahead) {
    (0, 0) => SyncScenario::UpToDate,
    (local_ahead, 0) => SyncScenario::PushOnly { local_ahead },
    (0, remote_ahead) => SyncScenario::PullOnly { remote_ahead },
    (local_ahead, remote_ahead) => SyncScenario::Diverged {
      local_ahead,
      remote_ahead,
    },
  }
}

pub fn parse_upstream_ref(value: &str) -> Option<(String, String)> {
  let value = value.trim();
  let (remote, branch) = value.split_once('/')?;
  Some((remote.to_string(), branch.to_string()))
}

pub fn parse_conflicted_files(output: &str) -> Vec<String> {
  output
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(str::to_string)
    .collect()
}

pub fn plural(count: usize) -> &'static str {
  if count == 1 { "" } else { "s" }
}
