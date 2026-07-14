---
layout: doc
part: 2
section: Storage Architecture
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Repository Model
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/repository-model/
---

## Repository Model

### Source of Truth

For every managed file:

1. The original file is moved from `$HOME` into the Tildr repository
2. A symlink is created at the original location in `$HOME`
3. Future reads and edits should happen against the repository-backed file

Example:

```text
$HOME/.config/nvim/init.lua            -> symlink
$HOME/.dotfiles/.config/nvim/init.lua  -> real file
```

### Default Paths

* Default repository: `~/.dotfiles`
* Config file: `~/.config/tildr/config.toml`
* If XDG config resolution is unavailable, Tildr falls back to `$HOME/.config/tildr/config.toml`

### Internal Directory

Tildr stores internal configuration files in a `.tildr/` directory inside the repository:

```text
.tildr/
├── encrypted-items    # manifest of registered sensitive files
├── encrypted.gpg      # encrypted bundle of sensitive files
└── groups.json        # named groups for batch operations
```

These files are managed automatically by Tildr and should not be edited manually.

### Filesystem Constraint

Tildr is designed to operate with the repository and managed files inside the same `$HOME` filesystem.

* The repository is required to live inside `$HOME`
* The repository cannot be `$HOME` itself
* Cross-disk repository layouts are not a supported workflow
* In practice, you should keep the Tildr repository inside your home directory and on the same filesystem as the files you manage

This means Tildr is not intended for copying files from your home directory to a repository stored on another disk or external mount. Keep the repository in `$HOME`.
