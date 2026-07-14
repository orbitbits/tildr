---
layout: doc
part: 1
section: Introduction
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: What is Tildr?
description: Manage and reproduce your HOME directory declaratively.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible, HOME-state]
permalink: /tildr/documentation/
---

## Introduction

**Manage, reproduce, and control your entire `$HOME`—declaratively.**

> **More powerful than *stow*. Simpler than *chezmoi*.**

**Tildr** is a fast, minimalist CLI for defining and reproducing your personal Unix environment.

Rather than manually copying dotfiles, syncing directories, or rebuilding your setup from memory, you describe the desired state of your `$HOME` in a declarative configuration. Tildr then ensures your system converges to that state safely and consistently.

Designed around simplicity, predictability, and idempotency, Tildr helps you keep your environment reproducible across new machines, reinstalls, and everyday changes.

---

## Why Tildr?

Traditional dotfile managers reproduce files. **Tildr** manages your **HOME state**.

Most dotfile managers treat your configuration as a collection of individual files. Tildr takes a broader view: your `$HOME` is an environment whose structure, contents, and behavior should be reproducible as a whole.

With **Tildr**, you can:

* Define the structure and contents of your `$HOME`
* Keep files and directories consistently in sync
* Recreate your environment reliably at any time
* Eliminate configuration drift
* Manage more than dotfiles—manage your **entire home state**

---

## Why the name?

The name **Tildr** is inspired by the **tilde** (`~`), one of the most recognizable symbols in Unix and Linux.

For decades, `~` has represented the user's **home directory**—a familiar starting point where configuration, files, and personal workflows naturally live. It's a small symbol with a meaning that every Unix user immediately understands.

That idea perfectly reflects the project's philosophy: your home directory is more than a place to store dotfiles—it's your personal environment.

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

## Overview

`Tildr` is a Rust CLI for managing files in your home directory on Linux and macOS through a repository-backed model.

Instead of keeping the original file in place, `Tildr` moves the managed file into a repository and creates a symlink back into `$HOME`. From that point on:

* The repository becomes the source of truth
* `$HOME` contains symlinks that represent the applied state
* `apply` re-creates or repairs those symlinks
* `restore` moves files back from the repository into `$HOME`
* `unlink` removes symlinks without deleting repository content
* `del` removes managed content from the repository and unlinks it from `$HOME`
* `open` opens the repository in the system file manager
* `stats` shows statistics about managed files
* `backup` creates a compressed tarball backup of the repository
* `suggest` scans `$HOME` for common dotfile patterns that could be managed
* `group` manages named groups of managed files for batch operations

`Tildr` manages files, not directories as first-class objects. Directory operations are recursive and act on all managed files under the selected path.

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

Initial setup and daily use:

```sh
tildr init
tildr suggest          # discover files to manage
tildr add .bashrc
tildr add .config/nvim
tildr git status
tildr status
tildr apply
tildr backup           # create a safety backup
tildr sync
```

Recovery and maintenance:

```sh
tildr status
tildr doctor
tildr apply
tildr unlink .config/nvim
tildr restore .bashrc
```

Secret file management:

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
* `git.auto_commit` affects `add`, `restore`, `del`, `mv`, and `secret` — not `apply`, `unlink`, `git`, or `sync`
* `git.enable = false` disables Tildr-managed Git operations even if Git is installed
* `tildr secret` requires `gpg` to be installed and available in `PATH`
* sensitive files registered with `tildr secret add` are never stored in plain text in the repository
* `crypto.mode` controls whether symmetric (passphrase) or asymmetric (key pair) GPG encryption is used
* in asymmetric mode, `crypto.gpg_key` is saved automatically after interactive key selection on first use
* `core.color = false` disables all colored output; `NO_COLOR` environment variable is also respected

---

## Summary

Tildr turns `$HOME` into a repository-backed declarative environment. Its model is simple:

* store real files in a repository
* expose them into `$HOME` through symlinks
* inspect drift with `status`
* converge state with `apply`
* recover ownership with `restore`

For reliable operation, keep the repository in your home directory, use `.tildrignore` to exclude unmanaged artifacts, and treat `tildr repo path` as the canonical way to locate the repository in scripts and shell aliases.
