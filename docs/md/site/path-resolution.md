---
layout: doc
part: 4
section: Path Handling
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Resolution Rules
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/path-resolution/
---

## Path Resolution Rules

Most path arguments are interpreted relative to `$HOME`.

* `tildr add .config/nvim/init.lua` resolves to `$HOME/.config/nvim/init.lua`
* `tildr add ~/notes/todo.md` resolves from the home shortcut
* Absolute paths are accepted only if they still point inside `$HOME`

For `init --repo`, the repository path may be provided as:

* `~/...` for a home-relative path
* an absolute path inside `$HOME`
* a relative path, which is resolved from the current working directory and must still end up inside `$HOME`
