---
layout: doc
part: 0
section: Quick Start
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Quick Start
description: Install and start using Tildr in minutes.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/quick-start/
---

## Quick Start

### Installation

**Linux — script (any distro):**

```sh
curl -fsSL https://orbitbits.com/tildr/linux.sh | sh
```

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

---

### First setup

```sh
# Initialize the Tildr repository
tildr init

# Discover files that could be managed
tildr suggest

# Add a file to manage
tildr add .bashrc

# Add a directory (traversed recursively)
tildr add .config/nvim

# Check the state of managed files
tildr status

# See what is in the repository
tildr list

# Verify the setup
tildr doctor

# Create a safety backup
tildr backup
```

---

### Everyday workflow

```sh
# Apply repository state to $HOME (repairs symlinks)
tildr apply

# Check for drift
tildr status

# See Git changes inside the repository
tildr git status

# View statistics about managed files
tildr stats

# Sync with a remote (bidirectional pull/push)
tildr sync
```

That is all you need to get started. See the [Command Reference](/tildr/documentation/commands/) for detailed usage of each command.
