# Changelog

All notable changes to this project will be documented in this file.

The format is based on Conventional Commits.

## [Unreleased] - 2026-07-11

### Bug Fixes

- installer: fix(installer): hide temp file path from sha256sum output ([d1e7c98](https://github.com/orbitbits/tildr/commit/d1e7c9812595dfe5c4162a49bdbc10bf07f3f368))
  - Extract only the hash via awk '{print $1}'
  - Display it through the info() helper instead of raw sha256sum output
  - Code formatting (indentation)

- fix: correcting value variables ([a291d17](https://github.com/orbitbits/tildr/commit/a291d17879565f1d0176be41e6184a03a88293dd))

### Features

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
