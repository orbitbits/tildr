---
layout: doc
part: 3
section: Setup & Options
menu: tildr
version: "0.2.0"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Configuration Reference
description: Complete configuration reference for Tildr.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.2.0/configuration/
---

## Configuration Reference

Tildr stores its user configuration in TOML format at `~/.config/tildr/config.toml`.

The configuration file is created by `tildr init` and is never written automatically by any other command. If the file does not exist, all defaults are applied silently at runtime.

---

### Configuration File Location

| Platform | Path |
|----------|------|
| Linux / macOS | `$XDG_CONFIG_HOME/tildr/config.toml` |
| Fallback | `$HOME/.config/tildr/config.toml` |

Tildr uses the XDG Base Directory specification when available. On systems where XDG is not configured, it falls back to `$HOME/.config/tildr/config.toml`.

---

### Full Configuration Example

```toml
[core]
repo = "~/.dotfiles"
search_threshold = 15
color = true

[git]
available = true
# enable = true          # optional: explicitly enable/disable Git operations
auto_commit = true

[crypto]
mode = "symmetric"
# gpg_key = ""           # only used when mode = "asymmetric"
```

---

### `[core]` Section

Core settings control the repository path, interactive behavior, and output formatting.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `repo` | `String` | `"~/.dotfiles"` | Path to the Tildr repository. Accepts `~/...` or an absolute path inside `$HOME`. |
| `search_threshold` | `Integer` | `15` | Number of managed files above which interactive pickers show a fuzzy search step before the selection list. |
| `color` | `Boolean` | `true` | When `false`, disables all colored output by setting `NO_COLOR=1` before dispatch. Also respected if `NO_COLOR` is already set in the environment. |

#### `core.repo`

The repository path must satisfy these constraints:

* Must be inside `$HOME`
* Cannot be `$HOME` itself
* Must be on the same filesystem as `$HOME` (no cross-disk layouts)

```toml
# Good
repo = "~/.dotfiles"
repo = "~/.config/dotfiles"
repo = "/home/user/.dotfiles"

# Bad — outside $HOME
repo = "/opt/dotfiles"

# Bad — is $HOME itself
repo = "~"
```

#### `core.search_threshold`

When the number of managed files exceeds this threshold, interactive pickers (used by `tildr add`, `tildr cat`, `tildr edit`, `tildr unlink`, `tildr restore`, `tildr del`, `tildr mv`) display a search input before the file list. Type a fragment to filter by fuzzy match, or press Enter with empty input to see the full list.

```toml
# Show search immediately for any number of files
search_threshold = 0

# Only show search for 50+ files
search_threshold = 50
```

#### `core.color`

Controls whether Tildr uses ANSI color codes in terminal output.

```toml
# Disable colors in output
color = false
```

Colors are also disabled when the `NO_COLOR` environment variable is set:

```sh
NO_COLOR=1 tildr status
```

---

### `[git]` Section

Git settings control version control integration, automatic commits, and the ability to disable Git operations entirely.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `available` | `Boolean` | `true` | Whether Git was detected by `tildr init`. Written automatically by Tildr. |
| `enable` | `Boolean` | *unset* | Optional override. When explicitly set to `false`, disables all Git operations even if Git is installed. |
| `auto_commit` | `Boolean` | `true` | When `true`, auto-runs `git add -A && git commit` after `add`, `restore`, `del`, `mv`, `secret`, `exclude`, `group`, and `profile` operations. |

#### `git.available`

This field is written automatically by `tildr init` based on whether Git was found in `PATH` at initialization time. You should not edit this field manually.

#### `git.enable`

This is an optional override that allows you to disable Git operations without uninstalling Git. When set to `false`:

* `tildr sync` will not work
* `tildr git status` will not work
* Auto-commit after `add`, `restore`, `del`, `mv`, `secret`, `exclude`, `group`, and `profile` will be skipped

```toml
[git]
# Explicitly disable all Git operations
enable = false
```

When unset (the default), Git operations are enabled if `git.available = true`.

#### `git.auto_commit`

When `true`, Tildr automatically commits changes after these commands:

| Command | Auto-commit behavior |
|---------|---------------------|
| `tildr add` | Commits after adding files |
| `tildr restore` | Commits after restoring files |
| `tildr del` | Commits after deleting files |
| `tildr mv` | Commits after moving/renaming files |
| `tildr secret add` | Commits after registering a secret file |
| `tildr secret rm` | Commits after unregistering a secret file |
| `tildr secret encrypt` | Commits after re-encrypting the bundle |
| `tildr exclude add` | Commits after adding an ignore pattern |
| `tildr exclude rm` | Commits after removing an ignore pattern |
| `tildr group create` | Commits after creating a group |
| `tildr group add` | Commits after adding files to a group |
| `tildr group rm` | Commits after removing files from a group |
| `tildr profile create` | Commits after creating a profile |
| `tildr profile add` | Commits after copying files between profiles |
| `tildr profile mv` | Commits after moving files between profiles |
| `tildr profile del` | Commits after deleting a profile |
| `tildr profile rename` | Commits after renaming a profile |
| `tildr profile set` | Commits after activating a profile |
| `tildr profile unset` | Commits after deactivating a profile |
| `tildr profile migrate` | Commits after migrating files to profiles model |

Commands that do **not** trigger auto-commit: `tildr apply`, `tildr unlink`, `tildr status`, `tildr list`, `tildr git`, `tildr sync`, `tildr doctor`.

```toml
[git]
# Disable automatic commits
auto_commit = false
```

#### Git Operations Logic

Tildr determines whether Git operations are enabled using this logic:

```text
operations_enabled  = git.available AND git.enable != false
auto_commit_enabled = git.auto_commit AND operations_enabled
```

---

### `[crypto]` Section

Encryption settings control how `tildr secret` encrypts and decrypts sensitive files.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mode` | `String` | `"symmetric"` | Encryption mode. Accepted values: `"symmetric"` or `"asymmetric"`. |
| `gpg_key` | `String` | `""` (empty) | GPG key ID or email for asymmetric mode. When empty, Tildr prompts on first use and saves the choice. |

#### `crypto.mode`

Tildr supports two GPG encryption modes:

**Symmetric** (default):

* No key pair required — only a passphrase
* GPG prompts for the passphrase via the system pinentry on first use
* The same passphrase must be used to decrypt on any machine
* Simpler setup, suitable for single-user environments

**Asymmetric**:

* Uses an existing GPG key pair — no separate passphrase to remember
* `crypto.gpg_key` must be set to the recipient key ID or email
* Decryption uses the private key silently (subject to GPG Agent caching)
* Preferred when you already manage GPG keys and want a seamless new-machine setup

```toml
[crypto]
# Symmetric mode (default)
mode = "symmetric"

# Asymmetric mode
mode = "asymmetric"
gpg_key = "william@email.com"
```

#### `crypto.gpg_key`

Only used when `crypto.mode = "asymmetric"`. Specifies the GPG key ID or email address used for encryption.

If left empty, Tildr will:

1. List all available GPG secret keys
2. If only one key exists, use it automatically
3. If multiple keys exist, show an interactive selection prompt
4. Save the chosen key to `config.toml` for future use

```toml
[crypto]
# By key ID
gpg_key = "ABC123DEF456"

# By email
gpg_key = "william@email.com"
```

---

### Configuration Loading Behavior

1. Tildr loads `config.toml` on startup
2. If the file does not exist, all defaults are applied silently
3. Missing fields within an existing file fall back to their defaults
4. The config is never written automatically except by `tildr init`
5. `tildr secret` may update `crypto.gpg_key` after interactive key selection

---

### Environment Variables

Tildr respects the following environment variable:

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | When set (any value), disables colored output. Equivalent to `core.color = false`. |
| `PAGER` | Used by `--less` flag in `tildr status`, `tildr list`, and `tildr cat`. Defaults to `less -RFX`. |

---

### Viewing Current Configuration

Use `tildr cat config` to display the current configuration file:

```sh
tildr cat config
```

This opens the config file in your configured editor, or prints it to stdout.
