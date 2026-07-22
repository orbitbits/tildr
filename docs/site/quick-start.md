---
layout: doc
part: 0
section: Quick Start
menu: tildr
version: "0.2.0"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Quick Start
description: Install and start using Tildr in minutes.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.2.0/quick-start/
---

## Quick Start

This guide covers installation, first-time setup, and everyday usage of Tildr.

---

### Prerequisites

| Requirement | Minimum        | Notes                                           |
|-------------|----------------|-------------------------------------------------|
| **OS**      | Linux or macOS | Windows is not currently supported              |
| **Git**     | 2.0+           | Optional but recommended for full functionality |
| **GPG**     | 2.0+           | Required only for `tildr secret` commands       |
| **Rust**    | 1.90.0+        | Required only for building from source          |

---

### Installation

**Linux — script (any distro):**

```sh
curl -fsSL https://orbitbits.com/tildr/linux.sh | sh
```

This script:

- Detects your distribution and installs the appropriate package
- Installs the Nautilus plugin if GNOME Files is detected
- Installs the Dolphin plugin if KDE Dolphin is detected

**macOS — script:**

```sh
curl -fsSL https://orbitbits.com/tildr/macos.sh | sh
```

**Debian / Ubuntu / Mint — apt repository:**

```sh
# Import GPG key
curl -fsSL https://deb.orbitbits.com/tildr-deb-pub.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/tildr.gpg

# Add repository
echo "deb [signed-by=/usr/share/keyrings/tildr.gpg] https://deb.orbitbits.com/ stable main" \
  | sudo tee /etc/apt/sources.list.d/tildr.list

# Install
sudo apt update && sudo apt install tildr
```

**Arch Linux — AUR (yay):**

```sh
yay -S tildr-bin
```

**Fedora / RHEL — RPM repository:**

```sh
# Import GPG key
sudo rpm --import https://rpm.orbitbits.com/tildr-rpm-pub.gpg

# Add repository
sudo dnf config-manager addrepo \
  --from-repofile=https://rpm.orbitbits.com/tildr.repo

# Install
sudo dnf install tildr
```

**From source (any platform):**

```sh
cargo install tildr
```

Or clone and build:

```sh
git clone https://github.com/orbitbits/tildr.git
cd tildr
cargo build --release
# Binary at target/release/tildr
```

---

### Verify Installation

```sh
tildr --version
tildr info credits
```

---

### First Setup

#### Step 1 — Initialize

```sh
tildr init
```

This creates:

- `~/.dotfiles/` — your Tildr repository
- `~/.config/tildr/config.toml` — your configuration file
- A Git repository inside `~/.dotfiles/` (if Git is available)

#### Step 2 — Discover Files

```sh
tildr suggest
```

Tildr scans `$HOME` for common dotfile patterns (shell configs, editor configs, terminal emulators, git, etc.) and suggests files that could be managed.

#### Step 3 — Add Files

```sh
# Add individual files
tildr add .bashrc
tildr add .zshrc .gitconfig .tmux.conf

# Add a directory (traversed recursively)
tildr add .config/nvim

# Add multiple files at once
tildr add .bashrc .zshrc .gitconfig
```

Each `add` command:

1. Moves the file from `$HOME` into `~/.dotfiles/`
2. Creates a symlink at the original location
3. Auto-commits to Git (if `git.auto_commit = true`)

#### Step 4 — Check Status

```sh
tildr status
```

Shows the synchronization state of files effective for the active profile:

| Status           | Meaning                                                        |
|------------------|----------------------------------------------------------------|
| `linked`         | Symlink exists and points to the correct repository file       |
| `missing_link`   | Repository file exists but the home symlink is absent          |
| `broken_symlink` | A symlink exists but points to a wrong target                  |
| `not_a_symlink`  | A regular file or directory exists where the symlink should be |

#### Step 5 — Verify Setup

```sh
tildr doctor
```

Runs health checks on:

- Repository existence
- Config file validity
- Git repository status
- File permissions
- Disk usage
- Symlink correctness

#### Step 6 — Create Backup

```sh
tildr backup
```

Creates a `.tar.gz` archive of the entire repository at `~/.dotfiles-backup-YYYY-MM-DD.tar.gz`.

---

### Everyday Workflow

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

# Open the repository in your file manager
tildr open
```

If your repository branch does not have an upstream configured, set the sync remote in `~/.config/tildr/config.toml`:

```toml
[git]
auto_commit = true
sync_remote = "origin"
sync_branch = "main"
```

With `auto_commit = true`, `tildr sync` commits pending repository changes before fetching, pulling, merging, or pushing. If `sync_branch` is omitted or empty, Tildr uses the current local branch name.

---

### Shell Aliases

Add these to your `~/.bashrc` or `~/.zshrc` for a smoother workflow:

```sh
# Jump to the Tildr repository
alias tcd='cd "$(tildr repo path)"'

# Quick status check
alias tstatus='tildr status --counter'

# Quick apply
alias tapply='tildr apply'

# Quick sync
alias tsync='tildr sync'

# Edit a managed file by name (no path needed)
alias tedit='tildr edit'

# Open the repository in your file manager
alias topen='tildr open'
```

These are suggestions — only add the ones you actually use. The key one is `tcd`, since a child process cannot change the parent shell's directory, `tildr repo cd` is not possible, but `cd "$(tildr repo path)"` works perfectly as an alias.

---

### Secret File Management

```sh
# Register a sensitive file
tildr secret add ~/.ssh/id_rsa

# List registered sensitive files
tildr secret list

# Manually re-encrypt
tildr secret encrypt

# Decrypt and restore sensitive files
tildr secret decrypt
```

Two encryption modes are available:

| Mode                    | Configuration                  | Passphrase Required                       |
|-------------------------|--------------------------------|-------------------------------------------|
| **Symmetric** (default) | `[crypto] mode = "symmetric"`  | Yes — same passphrase for encrypt/decrypt |
| **Asymmetric**          | `[crypto] mode = "asymmetric"` | No — uses GPG key pair                    |

---

### Batch Operations with Groups

```sh
# Create a group of related files
tildr group create dev --files .bashrc .zshrc .tmux.conf

# List all groups
tildr group list

# Apply all files in a group
tildr group rename dev shell

tildr group apply shell

# Remove symlinks for all files in a group
tildr group unlink shell
```

---

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

---

### Troubleshooting

**Repository must be inside $HOME:**

Tildr requires the repository to be inside your home directory. Move it:

```sh
mv ~/.dotfiles ~/dotfiles  # if it was outside $HOME
```

**Symlinks are broken after moving the repository:**

Run `tildr apply` to repair all symlinks:

```sh
tildr apply
```

**GPG not found:**

Install GPG for your platform:

```sh
# Debian/Ubuntu
sudo apt install gnupg

# Arch Linux
sudo pacman -S gnupg

# macOS
brew install gnupg
```

**File already managed:**

If a file is already managed by Tildr, running `tildr add` on it again is a no-op:

```sh
tildr add .bashrc    # First time — adds the file
tildr add .bashrc    # Second time — skipped (already managed)
```

**Interactive picker not showing:**

If the number of managed files is below `core.search_threshold` (default: `15`), the picker shows the full list directly. If it exceeds the threshold, a search step appears first.

**Auto-commit not working:**

Check your config:

```sh
tildr cat config
```

Ensure `[git] auto_commit = true` is set. Also ensure Git is available and `git.available = true`.

---

### Next Steps

- [Configuration Reference](/tildr/documentation/configuration/) — complete `config.toml` reference
- [Repository Model](/tildr/documentation/repository-model/) — how symlinks and Git work together
- [Commands Reference](/tildr/documentation/commands/) — detailed usage of each command
- [Secret Management](/tildr/documentation/secret-management/) — GPG encryption details
