---
layout: doc
part: 5
section: Filtering
menu: tildr
version: "0.2.0"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Ignore Patterns
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.2.0/ignore-support/
---

## Ignore Support

Tildr supports a repository-level `.tildrignore` file.

* The file must live at the root of the Tildr repository
* Ignore rules are applied when scanning repository contents
* Patterns use gitignore-style matching semantics

Example:

```text
*.log
cache/
.DS_Store
```

### What Tildr Ignores Internally

During repository scans, Tildr always excludes:

* `.git`
* `.gitignore`
* `.tildrignore`
* `.tildr/` — internal directory containing all Tildr configuration files
* `.DS_Store`
* `Thumbs.db`
* `.gitkeep`
* Files ending in `.bak`
* Files ending in `.tmp`
* Files ending in `.swp`
* Files ending in `~`
