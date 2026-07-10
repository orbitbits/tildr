---
title: TILDR
section: 1
header: User Commands
footer: Tildr
date: 2026
---

# NAME

tildr — manage, reproduce, and control everything in your $HOME declaratively

# SYNOPSIS

**tildr** *\<command\>* \[*options*\] \[*target*\]

# INTRODUCTION

**Tildr** is a fast and minimalist CLI that lets you define the desired state of your HOME directory and automatically enforce it.

Instead of manually copying dotfiles, syncing folders, or rebuilding your environment from memory, you describe how your `$HOME` should look — and **Tildr** makes your system converge to that state.

More powerful than *stow*. Simpler than *chezmoi*.

# WHY TILDR?

Traditional dotfile tools focus on files. **Tildr** focuses on **state**.

It treats your HOME directory as a reproducible environment — not just a collection of configs.

With **Tildr**, you can:

- Define the structure and contents of your `$HOME`
- Keep files and directories consistently in sync
- Recreate your environment reliably at any time
- Eliminate configuration drift
- Manage more than dotfiles — manage your **entire home state**

# PHILOSOPHY

Your `$HOME` should be:

**Declarative**
:   Defined by intent, not manual steps.

**Reproducible**
:   Rebuildable at any time.

**Consistent**
:   Always matching your desired state.

**Simple**
:   Without unnecessary complexity.

**Tildr** turns your HOME directory into a predictable and controlled environment.

# OVERVIEW

**Tildr** is a Rust CLI for managing files in your home directory on Linux and macOS through a repository-backed model.

Instead of keeping the original file in place, **Tildr** moves the managed file into a repository and creates a symlink back into `$HOME`. From that point on:

- The repository becomes the source of truth
- `$HOME` contains symlinks that represent the applied state
- `apply` re-creates or repairs those symlinks
- `restore` moves files back from the repository into `$HOME`
- `unlink` removes symlinks without deleting repository content
- `del` removes managed content from the repository and unlinks it from `$HOME`

**Tildr** manages files, not directories as first-class objects. Directory operations are recursive and act on all managed files under the selected path.

# REPOSITORY MODEL

For every managed file:

1. The original file is moved from `$HOME` into the Tildr repository
2. A symlink is created at the original location in `$HOME`
3. Future reads and edits should happen against the repository-backed file

Example:

```
$HOME/.config/nvim/init.lua           -> symlink
$HOME/.dotfiles/.config/nvim/init.lua -> real file
```

## Default Paths

**Default repository:**
:   `~/.dotfiles`

**Config file:**
:   `~/.config/tildr/config.toml`

If XDG config resolution is unavailable, Tildr falls back to `$HOME/.config/tildr/config.toml`.

## Filesystem Constraint

Tildr is designed to operate with the repository and managed files inside the same `$HOME` filesystem.

- The repository is required to live inside `$HOME`
- The repository cannot be `$HOME` itself
- Cross-disk repository layouts are not a supported workflow

This means Tildr is not intended for copying files to a repository stored on another disk or external mount. Keep the repository in `$HOME`.

# COMMANDS

**tildr init** *[options]*
:   Initialize the Tildr repository and config file.

**tildr import** *\<url\>* *[options]*
:   Clone a remote dotfiles repository and apply it.

**tildr add** *\<path...\>* *[options]*
:   Add files or directories to the repository and replace them with symlinks.

**tildr apply** *[options]*
:   Create or repair symlinks from repository to `$HOME`.

**tildr status** *[options]*
:   Show synchronization state of managed files.

**tildr list** *[options]*
:   List managed files in the repository.

**tildr repo**
:   Repository command group. Currently supports `tildr repo path`.

**tildr git** *\<mode\>*
:   Run Git-aware operations scoped to the Tildr repository.

**tildr cat** *[\<target\>]* *[options]*
:   Print the content of a managed file.

**tildr edit** *[\<target\>]*
:   Open a managed file in the configured editor.

**tildr unlink** *[\<target...\>]* *[options]*
:   Remove symlinks from `$HOME` without touching the repository.

**tildr restore** *[\<target...\>]* *[options]*
:   Move managed files back from the repository into `$HOME`.

**tildr del** *[\<target\>]* *[options]*
:   Delete managed files from the repository and unlink from `$HOME`.

**tildr mv** *[\<source\>]* *[\<dest\>]* *[options]*
:   Rename or move a managed file inside the repository.

**tildr exclude** *\<mode\>*
:   Manage `.tildrignore` patterns (add, remove, list).

**tildr secret** *\<mode\>*
:   Manage encryption of sensitive files using GPG.

**tildr sync** *[options]*
:   Synchronize the repository with its Git remote in both directions.

**tildr doctor**
:   Run a health check against the Tildr environment.

**tildr completions** *\<shell\>*
:   Generate shell completion scripts for bash, zsh, or fish.

**tildr info** *\<mode\>*
:   Display project metadata (credits or license).

See **tildr-commands(1)** for full documentation of each command.

# SEE ALSO

**tildr-commands(1)**, **tildr-config(1)**, **tildr-security(1)**, **tildr-plugins(1)**
