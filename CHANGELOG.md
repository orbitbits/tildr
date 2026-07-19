# Changelog

All notable changes to this project will be documented in this file.

The format is based on Conventional Commits.

## [0.2.1] - 2026-07-18

### Bug Fixes

- fix: show repo-relative paths in list, status, export, and interactive picker ([70f41ed](https://github.com/orbitbits/tildr/commit/70f41ed))

- fix: profile-aware file resolution in apply cat edit pick ([d6beb01](https://github.com/orbitbits/tildr/commit/d6beb01))

### Documentation

- docs(profile): improve active profile and bidirectional mv/add descriptions ([a73ee99](https://github.com/orbitbits/tildr/commit/a73ee99))

- docs: improve active profile and bidirectional mv/add documentation ([11b30fd](https://github.com/orbitbits/tildr/commit/11b30fd))

- docs: fix residual secret remove reference in secret-management.md ([122d4ec](https://github.com/orbitbits/tildr/commit/122d4ec))

## [0.2.0] - 2026-07-18

### Bug Fixes

- fix:  field assignment outside of initializer for an instance created with Default::default() ([c784f80](https://github.com/orbitbits/tildr/commit/c784f8061ae2683edb873a909364172423e280ee))

- fix: remove unwrap/expect, consolidate format_size, fix TOCTOU and paths bug ([82ff1d7](https://github.com/orbitbits/tildr/commit/82ff1d75c11fdb3378b53e50f5fc2bc6b01ea77a))

- profile: fix(profile): make status and doctor profile-aware ([641a88a](https://github.com/orbitbits/tildr/commit/641a88a4fd27e471eedd5ad98319a79fe53a9838))
  - status now uses profiles.resolve() to check correct symlink target
  - doctor SymlinkCheck now uses profiles.resolve() to check symlinks
  - Prevents false 'broken symlink' reports when profile is active

- group: fix(group): remove recursively when folder is passed to group remove ([1be9118](https://github.com/orbitbits/tildr/commit/1be9118ae5a51a0c26759050ceccc93af01a8591))
  'tildr group remove term --files .term' now removes all entries
  that start with '.term/' (recursive), not just exact matches.

- fix: resolve -l flag conflict in list and group apply error on existing symlinks ([0dd8ab3](https://github.com/orbitbits/tildr/commit/0dd8ab35834108ef1c18d5975ee1123e8d83da93))
  - Remove short flag from --less in tildr list (conflicted with --long)
  - group apply now skips existing correct symlinks instead of erroring
  - group apply shows summary (linked, up-to-date, skipped) like tildr apply

- hooks: fix(hooks): only check crates/ for unstaged build changes ([afac7ec](https://github.com/orbitbits/tildr/commit/afac7eccf5ea524b455b3b2805915676167a6900))
  cargo fmt only modifies .rs files in crates/. Checking git diff --quiet
  against the entire tree or --cached prevented individual commits when
  staging docs separately from code.

- hooks: fix(hooks): exclude hooks/ directory from pre-commit build check ([f669f17](https://github.com/orbitbits/tildr/commit/f669f1742614a6ddc7b1089ad6dcbcccd17c2251))
  The pre-commit hook checked git diff --cached --quiet which included
  the hooks/ directory itself, causing commits of hook changes to always
  fail. Now excludes hooks/ from the diff check.

- sync: fix(sync): re-encrypt only available files, skip missing ones ([c0b3168](https://github.com/orbitbits/tildr/commit/c0b316874c3b2dcbe1c58af3bb3c3fe4c73acc88))
  When running tildr sync, re_encrypt_before_push() now:
  - Filters out files that don't exist in HOME
  - Re-encrypts only the files that are available
  - Shows a warning listing which files were skipped
  - Skips entirely only if all files are missing

- sync: fix(sync): skip re-encryption when secret files are missing from HOME ([66b9a7a](https://github.com/orbitbits/tildr/commit/66b9a7ad2cfef5a71a254af690eb63a22045f476))
  When running tildr sync, re_encrypt_before_push() now checks if all
  registered files exist in HOME before attempting re-encryption.
  If any file is missing, it skips re-encryption and shows which files
  are not found, since the encrypted bundle already has the previous
  version.

- fix: ignore .tildr-groups.json in repository scanner ([db5ceda](https://github.com/orbitbits/tildr/commit/db5ceda97e7e6f78d715c82463ec0ffccaad706f))
  - Adds .tildr-groups.json to the internal ignore list
  - Prevents the groups file from appearing in tildr status, list, and apply
  - Consistent with how .tildr-encrypt and .tildrignore are handled

- installer: fix(installer): hide temp file path from sha256sum output ([d1e7c98](https://github.com/orbitbits/tildr/commit/d1e7c9812595dfe5c4162a49bdbc10bf07f3f368))
  - Extract only the hash via awk '{print $1}'
  - Display it through the info() helper instead of raw sha256sum output
  - Code formatting (indentation)

- fix: correcting value variables ([a291d17](https://github.com/orbitbits/tildr/commit/a291d17879565f1d0176be41e6184a03a88293dd))

### Features

- profile: feat(profile): add --long, --less, [NAME] to list; rename remove subcommands to rm ([a08bfdf](https://github.com/orbitbits/tildr/commit/a08bfdf14d7f1c1f57c7ed9458b314cffbd829f4))

- profile: feat(profile): add auto-copy workflow for profile add ([04ad039](https://github.com/orbitbits/tildr/commit/04ad03958b15002cceed573346ce9141c945d552))
  - 'tildr profile add work --files .bashrc' copies file to profiles/work/.bashrc
  - Folders are expanded recursively
  - Updated CLI help text with new examples
  - Updated documentation in site and man pages

- profile: feat(profile): add create/add/remove commands ([100e85d](https://github.com/orbitbits/tildr/commit/100e85d291ace1d8fd26d1810fdad3525ccf0fb1))
  - Add ProfileMode::Create with --description flag
  - Add ProfileMode::Add with --file and --variant flags
  - Add ProfileMode::Remove with --file flag
  - Update CLI help text with full examples
  - Update documentation in site and man pages

- profile: feat(profile): implement tildr profile command ([44da9bc](https://github.com/orbitbits/tildr/commit/44da9bc4f41bab71037b8483a3ce4364cf71a296))
  - Add ProfileMode enum (List, Set, Unset, Current) to domain layer
  - Create CLI definition with --help examples
  - Implement Profiles struct with load/save and resolve logic
  - Update apply command to use profile resolver for file variants
  - Add auto-commit on profile set/unset
  - Update documentation in site and man pages

- group: feat(group): auto-commit on group create/add/remove ([42ba795](https://github.com/orbitbits/tildr/commit/42ba795974dce51b686e3c551d019f10e307a014))
  'tildr group create', 'tildr group add', and 'tildr group remove'
  now auto-commit changes to the repository when git.auto_commit is
  enabled, consistent with 'tildr add' behavior.

- group: feat(group): expand folders recursively in group add --files ([83bfa0b](https://github.com/orbitbits/tildr/commit/83bfa0be30de2b7f2c9ce44914997f053031bb04))
  'tildr group add term --files .term' now detects if .term is a folder
  and expands it recursively, adding all files inside (e.g.
  .term/behavior.sh, .term/colors.sh). Files are added as individual
  entries in groups.json.

- group: feat(group): add file picker when --files is omitted in group add ([fbfd336](https://github.com/orbitbits/tildr/commit/fbfd3361cfad41cc48631e0b6373d8337ae5b0d5))
  - tildr group add `<name>` now opens a file picker in the repo
  - --files option still works for backwards compatibility
  - Folders are stored as single entry in groups.json (e.g. ['.term'])
  - Falls back to text input when no display is available

- feat: add tildr snapshot command for generating bootstrap scripts ([4b6184d](https://github.com/orbitbits/tildr/commit/4b6184d92b5475a3f1e6bc19095bf873a0b9e0ec))

- feat: add --less flag to tildr status and tildr list commands ([4b30ca8](https://github.com/orbitbits/tildr/commit/4b30ca8f544ef23552451ad41f5af725dbc56581))
  - Add --less flag to tildr status for interactive pager output
  - Add --less flag to tildr list for interactive pager output
  - Both commands now support piping output to less via  or less -RFX
  - Refactor list.rs helper functions to write to String buffer

- group: feat(group): add tildr group command for batch file management ([1bbd802](https://github.com/orbitbits/tildr/commit/1bbd802e4a7320f1b260c289fba5239b0093c423))
  - Create, add, remove, delete groups of managed files
  - Apply/unlink symlinks for entire groups at once
  - Groups stored in .tildr-groups.json in repository root
  - Supports create, add, remove, delete, list, apply, unlink subcommands

- suggest: feat(suggest): add tildr suggest command to suggest unmanaged dotfiles ([05b7e1d](https://github.com/orbitbits/tildr/commit/05b7e1d9e5f980639a2bf81a09e5667e52c87592))
  - Scans /home/boss for common dotfile patterns (shell, editor, terminal, git, tools)
  - Skips files already managed by Tildr
  - Reports suggestions grouped by category
  - Does not modify any files

- backup: feat(backup): add tildr backup command to create repository tarball ([b390fdd](https://github.com/orbitbits/tildr/commit/b390fdd4c2c46ab2f8c3685c973e31cdeb8b8987))
  - Creates compressed .tar.gz archive of the repository
  - Default output: ~/.dotfiles-backup-YYYY-MM-DD.tar.gz
  - Custom output path via --output flag
  - Adds chrono dependency for date formatting

- stats: feat(stats): add tildr stats command to show managed file statistics ([3dbae01](https://github.com/orbitbits/tildr/commit/3dbae012b95f7e69f1b8580ab8f9ea069d756bd9))
  - Shows total managed files, total size, largest file, and extension distribution
  - Uses scanner to read managed entries from repository
  - Adds console dependency for colored output

- open: feat(open): add tildr open command to open repo in file manager ([9aa43b2](https://github.com/orbitbits/tildr/commit/9aa43b2aa36b22d47ed08df9a6642a996fe132bc))
  - Uses the open crate (byron/open-rs) for cross-platform support
  - Opens the default file manager at the repository path
  - Adds open dependency to workspace and tildr-commands

- list: feat(list): add --export and --import flags ([47aaff8](https://github.com/orbitbits/tildr/commit/47aaff8d30a213350385bc8d6b470956d2b430c0))
  Add ability to export managed file list to JSON and import it on
  another machine. Export creates a portable snapshot with version,
  file paths. Import reads the JSON and creates symlinks in $HOME,
  skipping files already correctly linked.

- exclude: feat(exclude): auto-commit .tildrignore changes ([ee5357e](https://github.com/orbitbits/tildr/commit/ee5357ea6785fdb2ff28247f6239e966e96a7f9a))
  tildr exclude add and remove now auto-commit the .tildrignore file
  when git.auto_commit is enabled, consistent with other commands.

- feat: add tildr binary with full command system ([c1f57b3](https://github.com/orbitbits/tildr/commit/c1f57b300e63a68c391d13d1334f8ec84bdd5176))
  Add the main binary crate with all 20 commands:
  init, add, apply, status, list, repo, git, cat, edit,
  unlink, restore, del, mv, sync, import, secret, exclude,
  doctor, completions, info
  Includes interactive file pickers, auto-commit support,
  and shell completion generation for bash, zsh, and fish.

- feat: add new crate architecture ([cd9b0f5](https://github.com/orbitbits/tildr/commit/cd9b0f56682ad99f2e03e25ddd11a320b296b904))
  Restructure the workspace with focused crates replacing the monolithic
  tilder-core:
  - tildr-cli: CLI layer with clap-based command definitions
  - tildr-commands: Command implementations and dispatch logic
  - tildr-core: Configuration, context, and core utilities
  - tildr-crypto: GPG encryption/decryption for sensitive files
  - tildr-fs: Filesystem operations and symlink management
  - tildr-git: Git integration for repository operations
  - tildr-repo: Repository scanning, ignore support, and management
  - tildr-ui: Terminal output, colors, prompts, and symbols
  - tildr-utils: Shared utilities, macros, and helper functions

- feat: new design and variations brand Tilder ([eb8115e](https://github.com/orbitbits/tildr/commit/eb8115e84fba175b34517f8cea5cb8104b5ec945))

- installer: feat(installer): install to /usr/local/bin with sudo support ([a90e17e](https://github.com/orbitbits/tildr/commit/a90e17eb827237694cb45c8c629c046be4025dc9))
  - Change INSTALLATION_DIR from ~/.local/bin to /usr/local/bin
  - Add privilege helper: skip sudo if already root, otherwise require it
  - Prefix install/uninstall commands with $SUDO
  - Replace temp binary in $PWD with mktemp + trap for automatic cleanup

- installer: feat(installer): add version selection, --versions and --help flags, migrate to POSIX sh ([7b18629](https://github.com/orbitbits/tildr/commit/7b18629c053485e6f4a8b73b4a8ef360ede5dd0f))
  - Replace bash with POSIX sh (#!/usr/bin/env sh, set -e)
  - Add `<version>` argument to install any specific release
  - Add --versions flag to list all available GitHub releases
  - Add --help / -h flag with usage examples
  - Default behaviour (no args) still installs latest
  - Replace $EUID with $(id -u) for POSIX compliance
  - Replace bash arrays with plain for-in loop

- feat: add Rust workspace structure and core functionality ([10d7dac](https://github.com/orbitbits/tildr/commit/10d7dac8f34e0aa9560b2ab6d7091c990b666660))
  - Add workspace Cargo.toml with Rust 2024 edition and dependencies
  - Create tilder-core crate with cross-platform userprofile! macro
  - Add main tilder crate with CLI entry point and build.rs
  - Implement workspace metadata and build configuration
  - Add proper dependency management with Cargo.lock

### Refactoring

- test: refactor(test): extract inline tests into src/tests/ directories ([1b45032](https://github.com/orbitbits/tildr/commit/1b450328d692e44c9585cfbdf127e4cd10a1aa74))

- refactor: remove unused docs module ([6c7047d](https://github.com/orbitbits/tildr/commit/6c7047daf92d4e5a65eec0dc69a52be05d140ee2))

- commands: refactor(commands): remove duplicated confirm() in favor of tildr-utils ([27523ab](https://github.com/orbitbits/tildr/commit/27523abb2ac84505d1c5cf5459ad657d9c7452b5))

- utils: refactor(utils): extract shared operations to tildr-utils ([c2f8e67](https://github.com/orbitbits/tildr/commit/c2f8e67db28e45a98c5894971e5458860c959da5))
  - Move DeletionMode, ManagedPathOp, and cleanup_empty_ancestors to tildr-utils::ops
  - Add confirm utility to tildr-utils for interactive prompts
  - Update del, restore, and unlink commands to use shared modules
  - Add tildr-fs and tildr-ui dependencies to tildr-utils
  - Remove local ops.rs from tildr-commands/utils

- commands: refactor(commands): consolidate duplicated auto_commit and tildrignore logic ([f5bc7b7](https://github.com/orbitbits/tildr/commit/f5bc7b7903417f95c0e20400b73c62477f688ae5))
  - Create utils/auto_commit.rs with auto_commit() and auto_commit_dry_run()
  - Create utils/tildrignore.rs with append(), remove(), list() functions
  - Remove 8 duplicate auto_commit implementations from add, del, exclude,
  group, mv, profile, restore, and secret modules
  - Consolidate .tildrignore operations from add.rs and exclude.rs
  - Clean up exclude.rs redundant APP_NAME const and Context alias

- lint: refactor(lint): avoiding linter message (warning) ([3395edb](https://github.com/orbitbits/tildr/commit/3395edb2ae94a37aea5d7f6c6aeb1881cd97f7d0))

- refactor: migrate internal files to .tildr/ directory ([961092c](https://github.com/orbitbits/tildr/commit/961092cbad2c0dd728c25381b1a94af9d541672a))
  - Add tildr_dir() helper in tildr-utils/src/fs.rs
  - Move .tildr-encrypt → .tildr/encrypted-items
  - Move .tildr-encrypt.gpg → .tildr/encrypted.gpg
  - Move .tildr-groups.json → .tildr/groups.json
  - Update all crates to use tildr_dir() for internal paths
  - Update CLI help text and documentation

- docs: refactor(docs): improving the Tildr project description ([2271cb6](https://github.com/orbitbits/tildr/commit/2271cb6cf3a6f0451b59774687f1a3929836873e))

- refactor: rename tilder-core to tildr-domain ([260b5ba](https://github.com/orbitbits/tildr/commit/260b5ba5492e00e4f030e946ae36fbf06efae337))
  Rename the domain crate from tilder-core to tildr-domain to align
  with the project rebranding. The crate contains domain types,
  command definitions, and shared enums used across the workspace.

- refactor: remove old tilder binary crate ([aeb8480](https://github.com/orbitbits/tildr/commit/aeb8480f84d4993b1debbbc53825eda4ed79867d))
  Remove the legacy tilder/ directory containing the old binary crate.
  This crate is replaced by the new tildr/ binary with the restructured
  workspace architecture.

- refactor: removing the cliff function ([3d3e83a](https://github.com/orbitbits/tildr/commit/3d3e83a35adb2d3655c14b6e5eade7e26653ac78))

- refactor: new structure for the credits menu. ([0369248](https://github.com/orbitbits/tildr/commit/03692489efeb5e6295242b9c73ce92b983e92d8f))

- refactor: rename tilder_core to tilder-core following Rust naming conventions ([b2b1d7b](https://github.com/orbitbits/tildr/commit/b2b1d7beb1549c5cd2eec9d48a746528fcde9e0e))

- refactor: rename tilder-core to tilder_core following Rust convention ([927d3b8](https://github.com/orbitbits/tildr/commit/927d3b83eb3af30b2365f7f85aa36b4131986007))
