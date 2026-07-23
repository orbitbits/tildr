---
layout: doc
part: 4
section: Path Handling
menu: tildr
version: "0.3.2"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Resolution Rules
description: How Tildr resolves path arguments.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.3.2/path-resolution/
---

## Path Resolution Rules

Tildr resolves path arguments differently depending on the command context. Most commands interpret paths relative to `$HOME`, while `tildr init --repo` uses the current working directory.

---

### Commands That Resolve From `$HOME`

The following commands interpret path arguments relative to `$HOME`:

| Command | Resolution Base |
|---------|-----------------|
| `tildr add` | `$HOME` |
| `tildr restore` | `$HOME` |
| `tildr unlink` | `$HOME` |
| `tildr del` | `$HOME` |
| `tildr cat` | `$HOME` |
| `tildr edit` | `$HOME` |
| `tildr mv` | `$HOME` |
| `tildr secret add` | `$HOME` |

#### Relative Paths

Relative paths are interpreted from `$HOME`:

```sh
# These are equivalent:
tildr add .config/nvim/init.lua
tildr add ~/.config/nvim/init.lua

# Both resolve to:
# $HOME/.config/nvim/init.lua
```

#### Home Shortcut (`~`)

Paths starting with `~` are expanded to `$HOME`:

```sh
tildr add ~/notes/todo.md
# Resolves to: $HOME/notes/todo.md
```

#### Absolute Paths

Absolute paths are accepted only if they point inside `$HOME`:

```sh
# Valid — inside $HOME
tildr add /home/user/.bashrc

# Invalid — outside $HOME
tildr add /etc/hosts
# Error: path must be inside $HOME
```

---

### Path Resolution for `tildr init --repo`

The `--repo` flag for `tildr init` resolves paths differently:

| Input Type | Resolution |
|------------|------------|
| `~/...` | Expanded from `$HOME` |
| `/absolute/path` | Must be inside `$HOME` |
| `relative/path` | Resolved from current working directory, must end up inside `$HOME` |

#### Examples

```sh
# Home-relative
tildr init --repo ~/.dotfiles
# Creates: $HOME/.dotfiles

# Absolute (must be inside $HOME)
tildr init --repo /home/user/.dotfiles
# Creates: /home/user/.dotfiles

# Relative from CWD (must end up inside $HOME)
cd ~
tildr init --repo .dotfiles
# Creates: $HOME/.dotfiles

# Relative from CWD (invalid — ends up outside $HOME)
cd /tmp
tildr init --repo dotfiles
# Error: repository must be inside $HOME
```

---

### Path Resolution for `tildr secret add`

The `tildr secret add` command resolves paths from `$HOME`:

```sh
# These are equivalent:
tildr secret add ~/.ssh/id_rsa
tildr secret add .ssh/id_rsa

# Both resolve to:
# $HOME/.ssh/id_rsa
```

The file must exist in `$HOME` at the time of registration.

---

### Path Resolution for `tildr list --export` and `--import`

Export and import paths are resolved from the **current working directory**:

```sh
# Export to a file in the current directory
tildr list --export tildr-files.json

# Export to an absolute path
tildr list --export /tmp/tildr-files.json

# Import from a home-relative path
tildr list --import ~/tildr-files.json
```

---

### Path Resolution for `tildr backup --output`

The `--output` path is resolved from the **current working directory**:

```sh
# Create backup in current directory
tildr backup
# Creates: ~/.dotfiles-backup-YYYY-MM-DD.tar.gz

# Create backup in a specific location
tildr backup --output ~/backups/dotfiles.tar.gz
```

---

### Edge Cases

#### Spaces in Paths

Paths containing spaces must be quoted:

```sh
tildr add '.config/my app/config.toml'
tildr unlink '.config/my app/config.toml'
```

#### Symlinks as Arguments

If you pass a symlink path as an argument, Tildr follows the symlink to resolve the target:

```sh
# If ~/.current-shell is a symlink to ~/.zshrc
tildr add ~/.current-shell
# Adds: .zshrc (the symlink target)
```

#### Directories

Directory arguments are expanded recursively to effective active-profile files, with no-profile fallbacks, under that path:

```sh
# Adds all files under .config/nvim/
tildr add .config/nvim

# Unlinks all effective managed files under .config/
tildr unlink .config
```

#### Already Managed Files

If a file is already managed by Tildr, running `tildr add` on it again is a no-op:

```sh
tildr add .bashrc    # First time — adds the file
tildr add .bashrc    # Second time — skipped (already managed)
```

---

### Summary Table

| Command | Path Base | Accepts Absolute? | Accepts `~` / `$HOME`? |
|---------|-----------|-------------------|-------------------------|
| `tildr add` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr restore` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr unlink` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr del` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr cat` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr edit` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr mv` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr profile add/mv -f` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr secret add/rm` | `$HOME` | Yes (inside `$HOME`) | Yes |
| `tildr init --repo` | CWD | Yes (inside `$HOME`) | Yes |
| `tildr list --export` | CWD | Yes | Yes |
| `tildr list --import` | CWD | Yes | Yes |
| `tildr backup --output` | CWD | Yes | Yes |
| `tildr snapshot --output` | CWD | Yes | Yes |
