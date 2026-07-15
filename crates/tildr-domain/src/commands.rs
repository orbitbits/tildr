use crate::{GitMode, InfoMode, RepoMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretMode {
  Add { file: String },
  Remove { file: String },
  List,
  Encrypt,
  Decrypt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeMode {
  Add { pattern: String },
  Remove { pattern: String },
  List,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Commands {
  Init {
    repo: Option<String>,
    no_git: bool,
    force: bool,
    quiet: bool,
  },

  Add {
    paths: Option<Vec<String>>,
    force: bool,
    dry_run: bool,
    quiet: bool,
    nolink: bool,
  },

  Restore {
    targets: Vec<String>,
    all: bool,
    dry_run: bool,
    quiet: bool,
    force: bool,
  },

  Unlink {
    targets: Vec<String>,
    all: bool,
    dry_run: bool,
    quiet: bool,
    force: bool,
  },

  List {
    tree: bool,
    long: bool,
    export: Option<String>,
    import: Option<String>,
    less: bool,
  },

  Apply {
    dry_run: bool,
    force: bool,
    verbose: bool,
    quiet: bool,
  },

  Repo {
    mode: RepoMode,
  },

  Cat {
    target: Option<String>,
    less: bool,
  },

  Status {
    json: bool,
    counter: bool,
    less: bool,
  },

  Doctor,

  Completions {
    shell: String,
  },

  Edit {
    target: Option<String>,
  },
  Info {
    mode: InfoMode,
  },
  Git {
    mode: GitMode,
  },
  Sync {
    dry_run: bool,
    quiet: bool,
    force: bool,
  },
  Del {
    target: Option<String>,
    all: bool,
    dry_run: bool,
    quiet: bool,
    force: bool,
    purge: bool,
  },
  Import {
    url: String,
    dest: Option<String>,
    force: bool,
    quiet: bool,
    dry_run: bool,
  },
  Mv {
    source: Option<String>,
    dest: Option<String>,
    dry_run: bool,
    quiet: bool,
  },
  Secret {
    mode: SecretMode,
  },
  Exclude {
    mode: ExcludeMode,
  },
  Open,
  Stats,
  Backup {
    output: Option<String>,
  },
  Suggest,
  Snapshot {
    output: Option<String>,
  },
  Group {
    mode: GroupMode,
  },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupMode {
  Create {
    name: String,
    files: Vec<String>,
  },
  Add {
    name: String,
    files: Option<Vec<String>>,
  },
  Remove {
    name: String,
    files: Vec<String>,
  },
  Delete {
    name: String,
  },
  List,
  Apply {
    name: String,
  },
  Unlink {
    name: String,
  },
}
