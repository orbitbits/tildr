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

**Portable**
:   Move between machines effortlessly.

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
- `open` opens the repository in the configured or system file manager
- `stats` shows statistics about managed files
- `backup` creates a compressed tarball backup of the repository
- `suggest` scans `$HOME` for common dotfile patterns that could be managed
- `group` manages named groups of managed files for batch operations
- `profile` manages machine-specific dotfile variants for work, personal, laptop, etc.

**Tildr** manages files, not directories as first-class objects. Directory operations are recursive and act on the effective active-profile variants under the selected path; `--profile` targets another variant explicitly.

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

**Internal directory:**
:   `~/.dotfiles/.tildr/`

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

**tildr clean** *[options]*
:   Remove empty directories left inside profile storage.

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
:   Manage `.tildrignore` patterns (add, rm, list).

**tildr open**
:   Open the repository in the configured or system file manager.

**tildr stats**
:   Show statistics about managed files.

**tildr backup** *[options]*
:   Create a compressed tarball backup of the repository.

**tildr suggest**
:   Scan `$HOME` for common dotfile patterns that could be managed.

**tildr snapshot** *[options]*
:   Generate a reproducible bootstrap script from the current setup.

**tildr group** *\<mode\>*
:   Manage named groups of managed files for batch operations.

**tildr secret** *\<mode\>*
:   Manage encryption of sensitive files using GPG.

**tildr sync** *[options]*
:   Synchronize the repository with its Git remote in both directions.

**tildr doctor**
:   Run a health check against the Tildr environment.

**tildr completions** *\<shell\>*
:   Generate shell completion scripts for bash, zsh, fish, powershell, or elvish.

**tildr info** *\<mode\>*
:   Display project metadata (credits or license).

See **tildr-commands(1)** for full documentation of each command.

# INTERACTIVE BEHAVIOR

When a target is omitted, the following commands open an interactive file picker:

- `add` (picks from `$HOME` instead of the repository)
- `cat`
- `edit`
- `unlink`
- `restore`
- `del`
- `mv`

When the number of managed files exceeds `core.search_threshold` (default: `15`), the picker shows a search step first. Type a fragment to filter the list by fuzzy match, or press enter to skip filtering.

# TYPICAL WORKFLOW

Initial setup:

```sh
tildr init
tildr suggest
tildr add .bashrc
tildr add .config/nvim
tildr status
tildr doctor
tildr backup
```

Daily operations:

```sh
tildr apply
tildr apply --check
tildr status
tildr stats
tildr git status
tildr sync
```

For repositories without a local Git upstream, configure the sync remote and branch in
`~/.config/tildr/config.toml`:

```toml
[git]
auto_commit = true
sync_remote = "origin"
sync_branch = "main"
```

Recovery and maintenance:

```sh
tildr status
tildr doctor
tildr apply
tildr unlink .config/nvim
tildr restore .bashrc
```

Secret file management:

```sh
tildr secret add ~/.ssh/id_rsa
tildr secret list
tildr secret encrypt
tildr sync
```

Batch operations with groups:

```sh
tildr group create dev --files .bashrc .zshrc .tmux.conf
tildr group add term --files .term
tildr group rename dev shell

tildr group apply shell
tildr group unlink shell
```

Machine-specific profiles:

```sh
tildr profile create work --description "Work laptop"
tildr profile add no-profile --files .bashrc .ssh/config --to work
tildr profile set work

tildr profile rename personal desktop --description "Desktop dotfiles"

tildr profile set desktop

tildr profile unset
```

Bootstrap on a new machine:

```sh
tildr snapshot > setup.sh
chmod +x setup.sh
./setup.sh
```

# SHELL ALIASES

A child process cannot change the parent shell's working directory, so `tildr repo cd` is not possible. Use `tildr repo path` with aliases instead:

```sh
# Add to ~/.bashrc or ~/.zshrc
alias tcd='cd "$(tildr repo path)"'
alias tstatus='tildr status --counter'
alias tapply='tildr apply'
alias tsync='tildr sync'
```

See **tildr-commands(1)** for the full `tildr repo path` documentation.

# OPERATIONAL NOTES

- Tildr is designed for home-directory management on Linux and macOS
- The repository must stay inside `$HOME`
- Relative paths for managed targets are interpreted from `$HOME`
- Directory operations are recursive over effective active-profile variants under that path
- `apply` does not overwrite conflicting regular files unless `--force` is provided
- `unlink` removes only symlinks, never repository content
- `restore` physically moves the real file back out of the repository
- `del` removes repository content; use `--purge` for permanent deletion, otherwise files go to trash
- `git.auto_commit` affects mutating repository commands and lets `sync` commit pending changes before syncing — not `apply`, `unlink`, `git`, or read-only commands
- `tildr sync` uses the Git upstream when available, otherwise it falls back to `git.sync_remote` and `git.sync_branch`
- `git.enable = false` disables Tildr-managed Git operations even if Git is installed
- `tildr secret` requires `gpg` to be installed and available in `PATH`
- Sensitive files registered with `tildr secret add` are removed from Git tracking; only the encrypted bundle is committed
- `crypto.mode` controls whether symmetric (passphrase) or asymmetric (key pair) GPG encryption is used
- In asymmetric mode, `crypto.gpg_key` is saved automatically after interactive key selection on first use
- `core.color = false` disables all colored output; `NO_COLOR` environment variable is also respected
- The `--less` flag is available on `tildr status`, `tildr list`, and `tildr cat` for interactive pager output
- `.tildrignore` patterns prevent files from being discovered by `list`, `status`, and `apply`

# ENVIRONMENT VARIABLES

**EDITOR**
:   Editor used by `tildr edit`. Falls back to `$VISUAL`, then `nano`.

**VISUAL**
:   Alternative editor used by `tildr edit` if `$EDITOR` is not set.

**PAGER**
:   Pager used by `tildr cat --less` and `tildr status --less`. Falls back to `less -RFX`.

**NO_COLOR**
:   When set, disables all colored output regardless of `core.color` config.

**GPG_AGENT_INFO**
:   Used by GPG for passphrase caching in asymmetric mode.

# FILES

**~/.config/tildr/config.toml**
:   User configuration file. Created by `tildr init`.

**~/.dotfiles/**
:   Default repository location. Created by `tildr init`.

**~/.dotfiles/.tildr/**
:   Internal Tildr directory containing encrypted-items, encrypted.gpg, groups.json, and profiles.json.

**~/.dotfiles/.tildrignore**
:   User-defined ignore patterns for repository scanning.

# EXIT STATUS

**0**
:   Success.

**1**
:   General error (missing repository, invalid config, file not found).

**2**
:   Usage error (invalid arguments, unknown command).

# SEE ALSO

**tildr-commands(1)**, **tildr-config(1)**, **tildr-security(1)**, **tildr-plugins(1)**

# AUTHORS

Maintained by the Tildr contributors.
Source code and issue tracker: <https://github.com/orbitbits/tildr>
