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

Tildr supports a repository-level `.tildrignore` file for excluding paths from repository scans.

---

### How It Works

* The file must live at the **root** of the Tildr repository
* Ignore rules are applied when scanning repository contents
* Patterns use **gitignore-style** matching semantics
* The file is committed to the repository like any other managed file

---

### Creating a `.tildrignore`

You can create the file manually or use the `tildr exclude` command:

```sh
# Using the CLI
tildr exclude add *.log
tildr exclude add cache/
tildr exclude add .DS_Store

# Or create manually
echo "*.log" > ~/.dotfiles/.tildrignore
echo "cache/" >> ~/.dotfiles/.tildrignore
```

---

### Pattern Examples

#### Wildcard Patterns

```text
# Ignore all log files
*.log

# Ignore all backup files
*.bak

# Ignore all temporary files
*.tmp
```

#### Directory Patterns

```text
# Ignore the cache directory
cache/

# Ignore the .cache directory
.cache/

# Ignore all node_modules directories
**/node_modules/
```

#### Specific Files

```text
# Ignore a specific file
.DS_Store

# Ignore a specific directory
local/

# Ignore files by name
secret.key
passwords.txt
```

#### Negation Patterns

```text
# Ignore everything in .config
.config/*

# But keep these specific files
!.config/nvim/
!.config/starship.toml
```

#### Combined Patterns

```text
# Shell temporary files
*.swp
*~
*.bak

# OS-specific files
.DS_Store
Thumbs.db

# Build artifacts
target/
dist/
build/

# Editor temporary files
*.undo
.vscode/
.idea/
```

---

### Using `tildr exclude`

The `tildr exclude` command manages `.tildrignore` patterns without editing the file manually:

```sh
# Add a pattern
tildr exclude add *.log

# Remove a pattern
tildr exclude rm *.log

# List all patterns
tildr exclude list
```

**Behavior:**

* Creates `.tildrignore` if it does not exist
* Duplicate patterns are ignored
* Patterns added here prevent files from being discovered by `list`, `status`, and `apply`
* Does **not** remove existing symlinks — use `tildr unlink` for that

---

### What Tildr Ignores Internally

During repository scans, Tildr **always** excludes the following entries regardless of `.tildrignore`:

| Entry | Type | Reason |
|-------|------|--------|
| `.git` | Directory | Git repository data |
| `.gitignore` | File | Standard Git ignore rules |
| `.tildrignore` | File | Tildr ignore patterns |
| `.tildr/` | Directory | Internal Tildr configuration files |
| `.DS_Store` | File | macOS metadata |
| `Thumbs.db` | File | Windows thumbnail cache |
| `.gitkeep` | File | Placeholder for empty directories |
| `*.bak` | Files | Backup files |
| `*.tmp` | Files | Temporary files |
| `*.swp` | Files | Vim swap files |
| `*~` | Files | Editor backup files |

These entries are excluded from:
- `tildr list` output
- `tildr status` checks
- `tildr apply` symlink creation
- `tildr stats` calculations
- Interactive picker lists

---

### Common Use Cases

#### Ignore Editor Temporary Files

```text
*.swp
*.swo
*~
*.undo
.vscode/
.idea/
```

#### Ignore Build Artifacts

```text
target/
dist/
build/
*.o
*.so
*.dylib
```

#### Ignore OS-Specific Files

```text
.DS_Store
Thumbs.db
Desktop.ini
```

#### Ignore Secrets (without GPG)

```text
.ssh/id_rsa
.ssh/id_rsa.pub
.gnupg/
```

#### Ignore Cache Directories

```text
.cache/
.cache/**
npm-cache/
yarn/
```

---

### Relationship with `.gitignore`

Tildr's `.tildrignore` is separate from Git's `.gitignore`:

| File | Purpose | Scope |
|------|---------|-------|
| `.tildrignore` | Excludes files from Tildr operations | Tildr CLI |
| `.gitignore` | Excludes files from Git tracking | Git |

A file can be:
- Ignored by both (not tracked by Git, not managed by Tildr)
- Ignored by Tildr only (tracked by Git, not managed by Tildr)
- Ignored by Git only (managed by Tildr, not tracked by Git)
- Ignored by neither (managed by Tildr and tracked by Git)

---

### Verification

After adding patterns, verify they work:

```sh
# Check what Tildr sees
tildr list

# Check if specific files are excluded
tildr status

# Verify the ignore file
tildr exclude list
```

---

### Notes

* `.tildrignore` patterns only affect Tildr's repository scanning
* They do not affect Git tracking — use `.gitignore` for that
* Patterns are applied recursively to all subdirectories
* The `.tildrignore` file itself is always excluded from Tildr operations
* Internal exclusions (`.git`, `.tildr/`, etc.) cannot be overridden
