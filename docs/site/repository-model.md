---
layout: doc
part: 2
section: Storage Architecture
menu: tildr
version: "0.3.2"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Repository Model
description: How Tildr manages files through symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.3.2/repository-model/
---

## Repository Model

Tildr uses a **symlink-based** model where the Git repository serves as the single source of truth for all managed files.

---

### Source of Truth

For every managed file:

1. The original file is moved from `$HOME` into the Tildr repository
2. A symlink is created at the original location in `$HOME`
3. Future reads and edits happen against the repository-backed file

```text
Before tildr add:
$HOME/.config/nvim/init.lua    →  real file

After tildr add:
$HOME/.config/nvim/init.lua    →  symlink → $HOME/.dotfiles/.config/nvim/init.lua
$HOME/.dotfiles/.config/nvim/init.lua  →  real file
```

This model ensures that:

* The repository always contains the canonical version of each file
* `$HOME` reflects the applied state through symlinks
* `git diff` shows exactly what has changed
* `tildr status` detects any drift between desired and actual state

---

### How Symlinks Work

When you run `tildr add .config/nvim/init.lua`, Tildr:

1. **Moves** the file from `$HOME/.config/nvim/init.lua` to `~/.dotfiles/.config/nvim/init.lua`
2. **Creates** a symbolic link at `$HOME/.config/nvim/init.lua` pointing to the repository file
3. **Registers** the file in the repository's scan index

From this point:

* Editing `$HOME/.config/nvim/init.lua` edits the repository file (they are the same)
* `git status` in the repository shows changes to managed files
* `tildr apply` repairs the symlink if it gets deleted or broken
* `tildr restore` moves the file back to `$HOME` and removes the symlink

---

### Default Paths

| Path | Description |
|------|-------------|
| `~/.dotfiles` | Default repository location |
| `~/.config/tildr/config.toml` | Configuration file |
| `~/.dotfiles/.tildr/` | Internal Tildr directory (auto-managed) |
| `~/.dotfiles/.tildrignore` | User-defined ignore patterns |

---

### Internal Directory

Tildr stores internal configuration files in a `.tildr/` directory inside the repository:

```text
~/.dotfiles/.tildr/
├── encrypted-items    # manifest of registered sensitive files (plaintext)
├── encrypted.gpg      # encrypted bundle of sensitive files (GPG)
├── groups.json        # named groups for batch operations
└── profiles.json      # machine-specific profile definitions
```

#### `encrypted-items`

A plaintext manifest file listing the relative paths of all registered sensitive files, one per line. This file is committed to the repository.

```text
.ssh/id_rsa
.ssh/id_rsa.pub
.gnupg/private-keys-v1.d/ABC123.key
```

#### `encrypted.gpg`

An encrypted bundle containing all registered sensitive files packed into a tar archive and encrypted with GPG. This file is committed to the repository.

The bundle is re-encrypted automatically before `tildr sync` pushes changes to the remote.

#### `groups.json`

A JSON file storing named groups of managed files for batch operations. Groups are created and managed via `tildr group`.

```json
{
  "dev": [".bashrc", ".zshrc", ".tmux.conf"],
  "editor": [".config/nvim/init.lua", ".config/nvim/lua/plugins.lua"]
}
```

#### `profiles.json`

A JSON file storing machine-specific profile definitions. Profiles are created and managed via `tildr profile`.

```json
{
  "active": "work",
  "profiles": {
    "work": {
      "description": "Work laptop",
      "files": {
        ".bashrc": "profiles/work/.bashrc",
        ".ssh/config": "profiles/work/.ssh/config"
      }
    },
    "personal": {
      "description": "Personal desktop",
      "files": {
        ".bashrc": "profiles/personal/.bashrc"
      }
    }
  }
}
```

**Important:** These files are managed automatically by Tildr and should not be edited manually.

---

### Repository Structure

A typical Tildr repository looks like this:

```text
~/.dotfiles/
├── .git/                          # Git repository data
├── .gitignore                     # Git ignore rules
├── .tildrignore                   # Tildr ignore patterns
├── .tildr/                        # Internal Tildr directory
│   ├── encrypted-items            # Secret file manifest
│   ├── encrypted.gpg              # Encrypted bundle
│   ├── groups.json                # File groups
│   └── profiles.json              # Profile definitions
├── .bashrc                        # Managed file (real)
├── .zshrc                         # Managed file (real)
├── .gitconfig                     # Managed file (real)
├── .config/
│   ├── nvim/
│   │   ├── init.lua               # Managed file (real)
│   │   └── lua/
│   │       └── plugins.lua        # Managed file (real)
│   └── starship.toml              # Managed file (real)
├── profiles/                      # Profile variants
│   ├── work/
│   │   ├── .bashrc                # Work variant
│   │   └── .ssh/config            # Work variant
│   └── personal/
│       └── .bashrc                # Personal variant
└── .tmux.conf                     # Managed file (real)
```

And the corresponding `$HOME`:

```text
$HOME/
├── .bashrc                        → symlink → ~/.dotfiles/.bashrc
├── .zshrc                         → symlink → ~/.dotfiles/.zshrc
├── .gitconfig                     → symlink → ~/.dotfiles/.gitconfig
├── .config/
│   ├── nvim/
│   │   ├── init.lua               → symlink → ~/.dotfiles/.config/nvim/init.lua
│   │   └── lua/
│   │       └── plugins.lua        → symlink → ~/.dotfiles/.config/nvim/lua/plugins.lua
│   └── starship.toml              → symlink → ~/.dotfiles/.config/starship.toml
└── .tmux.conf                     → symlink → ~/.dotfiles/.tmux.conf
```

---

### What Gets Committed

Tildr commits the following to the repository:

| File | Committed? | Description |
|------|------------|-------------|
| Managed files | Yes | All files added via `tildr add` |
| `.tildr/encrypted-items` | Yes | Secret file manifest |
| `.tildr/encrypted.gpg` | Yes | Encrypted bundle |
| `.tildr/groups.json` | Yes | File groups |
| `.tildrignore` | Yes | User-defined ignore patterns |
| `.gitignore` | Yes | Standard Git ignore rules |

Root `.gitignore` and `.tildrignore` are repository control files. The same
filenames under `common/` or `profiles/<name>/` are managed HOME dotfiles, so
`~/.gitignore` can be tracked without colliding with the repository's own
ignore configuration.

What does **not** get committed:

| File | Committed? | Description |
|------|------------|-------------|
| `.git/` | No | Git repository data |
| Files matching `.tildrignore` | No | Excluded by user patterns |
| Files in `.tildr/` (temp) | No | Temporary encryption files |
| Sensitive files (raw) | No | Only the encrypted bundle is committed |

---

### Filesystem Constraints

Tildr is designed to operate with the repository and managed files inside the same `$HOME` filesystem.

* The repository is required to live inside `$HOME`
* The repository cannot be `$HOME` itself
* Cross-disk repository layouts are not a supported workflow
* In practice, you should keep the Tildr repository inside your home directory and on the same filesystem as the files you manage

This means Tildr is not intended for copying files from your home directory to a repository stored on another disk or external mount. Keep the repository in `$HOME`.

---

### Symbolic Link Behavior

Tildr creates **absolute symlinks** by default. Each symlink points to the full path of the file in the repository:

```sh
readlink ~/.bashrc
# Output: /home/user/.dotfiles/.bashrc
```

If the repository is moved, all symlinks will break. Use `tildr apply` to repair them after moving the repository.

---

### Idempotency

Tildr operations are designed to be idempotent — running the same command multiple times produces the same result:

* `tildr apply` only creates missing symlinks; existing correct symlinks are skipped
* `tildr add` only adds files that are not already managed
* `tildr unlink` only removes symlinks that exist
* `tildr restore` only restores files that are in the repository

This means you can safely run any command multiple times without side effects.
