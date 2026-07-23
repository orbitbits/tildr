---
layout: doc
part: 1
section: Introduction
menu: tildr
version: "0.3.1"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: What is Tildr?
description: Manage and reproduce your HOME directory declaratively.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible, HOME-state]
permalink: /tildr/documentation/0.3.1/
---

## Introduction

**Manage, reproduce, and control your entire `$HOME` — declaratively.**

> **More powerful than *stow*. Simpler than *chezmoi*.**

**Tildr** is a fast, minimalist CLI for defining and reproducing your personal Unix environment.

Rather than manually copying dotfiles, syncing directories, or rebuilding your setup from memory, you describe the desired state of your `$HOME` in a declarative configuration. Tildr then ensures your system converges to that state safely and consistently.

Designed around simplicity, predictability, and idempotency, Tildr helps you keep your environment reproducible across new machines, reinstalls, and everyday changes.

---

### Key Features

| Feature                  | Description                                                                     |
|--------------------------|---------------------------------------------------------------------------------|
| **Symlink-based model**  | Real files live in a Git repository; `$HOME` contains symlinks pointing to them |
| **Git integration**      | Automatic commits, bidirectional sync, and full version history                 |
| **Secret management**    | GPG encryption for sensitive files (SSH keys, GPG keys, credentials)            |
| **Interactive pickers**  | Fuzzy file selection when no target is specified                                |
| **Cross-platform**       | Works on Linux and macOS with consistent behavior                               |
| **Zero-config defaults** | Works out of the box; configure only what you need                              |
| **Auto-commit**          | Optional automatic Git commits after file operations                            |
| **Backup & restore**     | Create tarball backups, restore files from repository                           |
| **File groups**          | Batch operations on named groups of managed files                               |
| **Machine profiles**     | Different dotfile variants for work, personal, laptop, etc.                    |
| **File suggestions**     | Scan `$HOME` for common dotfile patterns that could be managed                  |

---

## Why Tildr?

Traditional dotfile managers reproduce files. **Tildr** manages your **HOME state**.

Most dotfile managers treat your configuration as a collection of individual files. Tildr takes a broader view: your `$HOME` is an environment whose structure, contents, and behavior should be reproducible as a whole.

With **Tildr**, you can:

* Define the structure and contents of your `$HOME`
* Keep files and directories consistently in sync
* Recreate your environment reliably at any time
* Eliminate configuration drift
* Manage more than dotfiles — manage your **entire home state**

---

## Why the name?

The name **Tildr** is inspired by the **tilde** (`~`), one of the most recognizable symbols in Unix and Linux.

For decades, `~` has represented the user's **home directory** — a familiar starting point where configuration, files, and personal workflows naturally live. It's a small symbol with a meaning that every Unix user immediately understands.

That idea perfectly reflects the project's philosophy: your home directory is more than a place to store dotfiles — it's your personal environment.

Rather than using *Tilde* directly, the name was distilled into **Tildr**: shorter, more distinctive, and better suited as a modern software project while preserving its Unix roots.

For experienced Unix users, it's a subtle nod to a symbol they've used countless times. For everyone else, it's simply a memorable name that grows with the project.

---

## Philosophy

Your `$HOME` should be:

* **Declarative** — defined by intent, not manual steps
* **Reproducible** — rebuildable at any time
* **Consistent** — always matching your desired state
* **Simple** — without unnecessary complexity
* **Portable** — move between machines effortlessly

`Tildr` turns your HOME directory into a predictable and controlled environment.

---

## Architecture

Tildr follows a three-layer architecture:

```text
┌─────────────────────────────────────────────────────────┐
│                      CLI Layer                          │
│              (tildr-cli + clap)                          │
│         Argument parsing, help text, completions         │
├─────────────────────────────────────────────────────────┤
│                     Domain Layer                         │
│                (tildr-domain)                            │
│           Command variants, type definitions             │
├─────────────────────────────────────────────────────────┤
│                   Commands Layer                         │
│               (tildr-commands)                           │
│          Business logic, file operations                 │
├─────────────────────────────────────────────────────────┤
│                    Core Layer                            │
│     (tildr-core, tildr-fs, tildr-git, tildr-repo)      │
│        Config, filesystem, Git, repository               │
├─────────────────────────────────────────────────────────┤
│                   Utils Layer                            │
│            (tildr-utils, tildr-ui)                       │
│          Formatting, pager, color, icons                 │
└─────────────────────────────────────────────────────────┘
```

### Crate Structure

| Crate            | Purpose                                        |
|------------------|------------------------------------------------|
| `tildr`          | Binary entry point                             |
| `tildr-cli`      | Clap-based CLI definitions and completions     |
| `tildr-domain`   | Language-agnostic command variants and enums   |
| `tildr-commands` | Business logic for all commands                |
| `tildr-core`     | Configuration loading, context, error types    |
| `tildr-fs`       | Symlink operations, path utilities             |
| `tildr-git`      | Git integration (auto-commit, status)          |
| `tildr-repo`     | Repository scanning and managed file discovery |
| `tildr-crypto`   | GPG encryption, manifest management            |
| `tildr-utils`    | Formatting, pager, filesystem helpers          |
| `tildr-ui`       | Terminal output, colors, icons, prompts        |
| `open`           | Cross-platform file manager launching          |
| `chrono`         | Date formatting for backup timestamps          |

---

## Overview

`Tildr` is a Rust CLI for managing files in your home directory on Linux and macOS through a repository-backed model.

Instead of keeping the original file in place, `Tildr` moves the managed file into a repository and creates a symlink back into `$HOME`. From that point on:

* The repository becomes the source of truth
* `$HOME` contains symlinks that represent the applied state
* `apply` re-creates or repairs those symlinks
* `clean` removes empty directories left in profile storage
* `restore` moves files back from the repository into `$HOME`
* `unlink` removes symlinks without deleting repository content
* `del` removes managed content from the repository and unlinks it from `$HOME`
* `open` opens the repository in the configured or system file manager
* `stats` shows statistics about managed files
* `backup` creates a compressed tarball backup of the repository
* `suggest` scans `$HOME` for common dotfile patterns that could be managed
* `group` manages named groups of managed files for batch operations
* `profile` manages machine-specific dotfile variants for work, personal, laptop, etc.

`Tildr` manages files, not directories as first-class objects. Directory operations are recursive and act on effective active-profile variants under the selected path; `--profile` targets another profile explicitly.

---

## Interactive Behavior

When a target is omitted, the following commands open an interactive file picker over the list of managed files:

* `tildr add` (picks from `$HOME` instead of the repository)
* `tildr cat`
* `tildr edit`
* `tildr unlink`
* `tildr restore`
* `tildr del`
* `tildr mv`

The picker operates on the managed files discovered by scanning the repository.

When the number of managed files exceeds `core.search_threshold` (default: `15`), the picker shows a search step first. Type a fragment to filter the list by fuzzy match, or press enter with an empty input to skip filtering and see the full list.

---

## Typical Workflow

### Initial Setup

```sh
# Initialize the repository
tildr init

# Discover files that could be managed
tildr suggest

# Add files to manage
tildr add .bashrc
tildr add .config/nvim
tildr add .zshrc .gitconfig .tmux.conf

# Check the state of managed files
tildr status

# See what is in the repository
tildr list

# Verify the setup
tildr doctor

# Create a safety backup
tildr backup
```

### Daily Operations

```sh
# Apply repository state to $HOME (repairs symlinks)
tildr apply

# Check for drift
tildr status

# View statistics about managed files
tildr stats

# See Git changes inside the repository
tildr git status

# Sync with a remote (bidirectional pull/push)
tildr sync
```

For repositories without a local Git upstream, configure the remote and branch explicitly:

```toml
[git]
auto_commit = true
sync_remote = "origin"
sync_branch = "main"
```

### Recovery and Maintenance

```sh
# Check status and diagnose issues
tildr status
tildr doctor

# Repair broken symlinks
tildr apply

# Remove symlinks without deleting files
tildr unlink .config/nvim

# Move files back from repository to $HOME
tildr restore .bashrc

# Delete managed files permanently
tildr del .config/nvim --purge
```

### Secret File Management

```sh
# Symmetric mode (default) — uses a passphrase
tildr secret add ~/.ssh/id_rsa
tildr secret list
tildr secret encrypt
tildr sync

# Asymmetric mode — set in config.toml first
# [crypto]
# mode = "asymmetric"
# gpg_key = "william@email.com"
tildr secret add ~/.ssh/id_rsa
tildr sync
```

### Batch Operations with Groups

```sh
# Create a group of related files
tildr group create dev --files .bashrc .zshrc .tmux.conf

# Apply all files in a group
tildr group rename dev shell

tildr group apply shell

# Remove symlinks for all files in a group
tildr group unlink shell

# List all groups
tildr group list
```

### Machine-specific Profiles

```sh
# Create a profile for work environment
tildr profile create work --description "Work laptop"

# Add files to the profile (copies to profiles/work/)
tildr profile add no-profile --files .bashrc .ssh/config --to work

# Create another profile
tildr profile create personal --description "Personal desktop"
tildr profile add no-profile --files .bashrc .gitconfig --to personal

# Activate a profile and relink matching dotfiles immediately
tildr profile set work

# Switch profiles and relink again
tildr profile rename personal desktop --description "Desktop dotfiles"

tildr profile set desktop

# Deactivate and relink to shared no-profile files
tildr profile unset
```

---

## Comparison with Other Tools

| Feature                 | Tildr         | GNU Stow      | chezmoi                 | yadm           |
|-------------------------|---------------|---------------|-------------------------|----------------|
| **Model**               | Symlinks      | Symlinks      | Templates + symlinks    | Git + symlinks |
| **Language**            | Rust          | Perl          | Go                      | Shell          |
| **Interactive pickers** | Yes           | No            | No                      | No             |
| **Secret management**   | Built-in GPG  | No            | Built-in age/GPG        | External       |
| **Auto-commit**         | Yes           | No            | Optional                | No             |
| **File groups**         | Yes           | No            | No                      | No             |
| **Machine profiles**    | Yes           | No            | Templates               | No             |
| **Suggest unmanaged**   | Yes           | No            | No                      | No             |
| **Cross-platform**      | Linux + macOS | Linux + macOS | Linux + macOS + Windows | Linux + macOS  |
| **Backup**              | Built-in      | No            | No                      | No             |

---

## Operational Notes

* Tildr is designed for home-directory management on Linux and macOS
* The repository must stay inside `$HOME`
* Relative paths for managed targets are interpreted from `$HOME`
* Directory operations are always recursive over files under that path
* `apply` does not overwrite conflicting regular files unless `--force` is provided
* `unlink` removes only symlinks, never repository content
* `restore` physically moves the real file back out of the repository
* `del` removes repository content; use `--purge` for permanent deletion, otherwise files go to trash
* `git.auto_commit` affects mutating repository commands and lets `sync` commit pending changes before syncing — not `apply`, `unlink`, `git`, or read-only commands
* `tildr sync` uses the Git upstream when available, otherwise it falls back to `git.sync_remote` and `git.sync_branch`
* `git.enable = false` disables Tildr-managed Git operations even if Git is installed
* `tildr secret` requires `gpg` to be installed and available in `PATH`
* Sensitive files registered with `tildr secret add` are removed from Git tracking; only the encrypted bundle is committed
* `crypto.mode` controls whether symmetric (passphrase) or asymmetric (key pair) GPG encryption is used
* In asymmetric mode, `crypto.gpg_key` is saved automatically after interactive key selection on first use
* `core.color = false` disables all colored output; `NO_COLOR` environment variable is also respected
* The `--less` flag is available on `tildr status`, `tildr list`, and `tildr cat` for interactive pager output

---

## Summary

Tildr turns `$HOME` into a repository-backed declarative environment. Its model is simple:

* Store real files in a Git repository
* Expose them into `$HOME` through symlinks
* Inspect drift with `status`
* Converge state with `apply`
* Recover ownership with `restore`
* Encrypt sensitive files with `secret`
* Sync across machines with `sync`

For reliable operation, keep the repository in your home directory, use `.tildrignore` to exclude unmanaged artifacts, and treat `tildr repo path` as the canonical way to locate the repository in scripts and shell aliases.

---

## Shell Aliases

A child process (CLI) cannot change the parent shell's working directory. This means `tildr repo cd` is not technically possible. Instead, use `tildr repo path` with shell aliases:

```sh
# Add to ~/.bashrc or ~/.zshrc

# Jump to the Tildr repository
alias tcd='cd "$(tildr repo path)"'

# Quick status check
alias tstatus='tildr status --counter'

# Quick apply
alias tapply='tildr apply'

# Quick sync
alias tsync='tildr sync'

# Edit a managed file by name
alias tedit='tildr edit'

# Open the repository in your file manager
alias topen='tildr open'
```

These are suggestions — only add the ones you actually use.
