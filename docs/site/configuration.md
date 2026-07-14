---
layout: doc
part: 3
section: Setup & Options
menu: tildr
version: "0.2.0"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Configuration Reference
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.2.0/configuration/
---

## Configuration

Tildr stores its user configuration in TOML at `~/.config/tildr/config.toml`.

```toml
[core]
repo = "~/.dotfiles"
search_threshold = 15
color = true

[git]
available = true
auto_commit = true

[crypto]
mode = "symmetric"
# gpg_key = "william@email.com"   # only used when mode = "asymmetric"
```

### Supported Keys

* `core.repo` — repository path used by the CLI. Accepts `~/...` or an absolute path inside `$HOME`. Default: `~/.dotfiles`
* `core.search_threshold` — number of managed files above which interactive pickers show a search/filter step before the selection list. Default: `15`
* `core.color` — when `false`, disables all colored output by setting `NO_COLOR=1` before dispatch. Also respected if `NO_COLOR` is already set in the environment. Default: `true`
* `git.available` — whether Git was detected by `tildr init`. This value is written automatically by Tildr and used by Git-aware commands. Default when no config exists: `true`
* `git.enable` — optional override. When explicitly set to `false`, Tildr skips Git operations even if Git is installed. Default: unset
* `git.auto_commit` — when `true`, Tildr automatically runs `git add -A` and `git commit` after `add`, `restore`, and `del`, but only when Git operations are enabled. Default: `true`
* `crypto.mode` — encryption mode used by `tildr secret`. Accepted values: `symmetric` (passphrase only) or `asymmetric` (GPG key pair). Default: `symmetric`
* `crypto.gpg_key` — GPG key ID or email address used when `crypto.mode = "asymmetric"`. When empty, Tildr prompts interactively on first use and saves the chosen key. Default: empty

### Configuration Loading

Tildr loads `config.toml` on startup. If the file does not exist, all defaults are applied silently. The config is never written automatically except by `tildr init`.
