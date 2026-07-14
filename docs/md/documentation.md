---
layout: doc
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Tildr
description: "Manage and reproduce your HOME directory declaratively."
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
update_date:
published: false
permalink: /tildr/documentation/
---

## Introduction

**Manage, reproduce, and control your entire `$HOME`—declaratively.**

> **More powerful than *stow*. Simpler than *chezmoi*.**

**Tildr** is a fast, minimalist CLI for defining and reproducing your personal Unix environment.

Instead of manually copying dotfiles, syncing directories, or rebuilding your setup from memory, you describe the desired state of your `$HOME` in a declarative configuration. Tildr then ensures your system converges to that state safely and consistently.

Designed around simplicity, predictability, and idempotency, Tildr helps you keep your environment reproducible across new machines, reinstalls, and everyday changes.

---

## Why Tildr?

Traditional dotfile tools manage files. **Tildr** manages your **HOME state**.

Most dotfile managers treat your configuration as a collection of individual files. Tildr takes a broader view: your `$HOME` is an environment whose structure, contents, and behavior should be reproducible as a whole.

With **Tildr**, you can:

* Define the structure and contents of your `$HOME`
* Keep files and directories consistently in sync
* Recreate your environment reliably at any time
* Eliminate configuration drift
* Manage more than dotfiles—manage your **entire home state**

---

## Why the name?

The name **Tildr** is inspired by the **tilde** (`~`), one of the most recognizable symbols in Unix and Linux.

For decades, `~` has represented the user's **home directory**—a familiar starting point where configuration, files, and personal workflows naturally live. It's a small symbol with a meaning that every Unix user immediately understands.

That idea perfectly reflects the project's philosophy: your home directory is more than a place to store dotfiles—it's your personal environment.

Rather than using *Tilde* directly, the name evolved into **Tildr**: shorter, more distinctive, and better suited as a modern software project while preserving its Unix roots.

For experienced Unix users, it's a subtle nod to a symbol they've used countless times. For everyone else, it's simply a memorable name that grows with the project.

---

## Philosophy

Your `$HOME` should be:

* **Declarative** — defined by intent, not manual steps
* **Reproducible** — rebuildable at any time
* **Consistent** — always matching your desired state
* **Simple** — without unnecessary complexity

`Tildr` turns your HOME directory into a predictable and controlled environment.

---

## Overview

`Tildr` is a Rust CLI for managing files in your home directory on Linux and macOS through a repository-backed model.

Instead of keeping the original file in place, `Tildr` moves the managed file into a repository and creates a symlink back into `$HOME`. From that point on:

* The repository becomes the source of truth
* `$HOME` contains symlinks that represent the applied state
* `apply` re-creates or repairs those symlinks
* `restore` moves files back from the repository into `$HOME`
* `unlink` removes symlinks without deleting repository content
* `del` removes managed content from the repository and unlinks it from `$HOME`

`Tildr` manages files, not directories as first-class objects. Directory operations are recursive and act on all managed files under the selected path.

---

## Repository Model

### Source of Truth

For every managed file:

1. The original file is moved from `$HOME` into the Tildr repository
2. A symlink is created at the original location in `$HOME`
3. Future reads and edits should happen against the repository-backed file

Example:

```text
$HOME/.config/nvim/init.lua            -> symlink
$HOME/.dotfiles/.config/nvim/init.lua  -> real file
```

### Default Paths

* Default repository: `~/.dotfiles`
* Config file: `~/.config/tildr/config.toml`
* If XDG config resolution is unavailable, Tildr falls back to `$HOME/.config/tildr/config.toml`

### Filesystem Constraint

Tildr is designed to operate with the repository and managed files inside the same `$HOME` filesystem.

* The repository is required to live inside `$HOME`
* The repository cannot be `$HOME` itself
* Cross-disk repository layouts are not a supported workflow
* In practice, you should keep the Tildr repository inside your home directory and on the same filesystem as the files you manage

This means Tildr is not intended for copying files from your home directory to a repository stored on another disk or external mount. Keep the repository in `$HOME`.

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

---

## Path Resolution Rules

Most path arguments are interpreted relative to `$HOME`.

* `tildr add .config/nvim/init.lua` resolves to `$HOME/.config/nvim/init.lua`
* `tildr add ~/notes/todo.md` resolves from the home shortcut
* Absolute paths are accepted only if they still point inside `$HOME`

For `init --repo`, the repository path may be provided as:

* `~/...` for a home-relative path
* an absolute path inside `$HOME`
* a relative path, which is resolved from the current working directory and must still end up inside `$HOME`

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

---

## Plugins

Tildr provides native integration with popular Linux file managers through plugins,
allowing you to manage your files directly from the context menu without using the terminal.

### Nautilus

Integration with Nautilus (GNOME Files) is done via **nautilus-python**,
a Python extension API that allows adding custom items to the context menu.

**Requirements:**

* `python-nautilus` (Arch Linux)
* `python3-nautilus` (Debian / Ubuntu / Mint)
* `nautilus-python` (Fedora)

**Features:**

* Context menu with submenu **Tildr → Add / Unlink / Restore**
* Supports single and multiple file selection
* Available only for files and symlinks (not directories)
* Only appears inside the user's home directory (`~/`)

**Manual installation:**

```sh
mkdir -p ~/.local/share/nautilus/python-extensions
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/nautilus/tildr.py \
  -o ~/.local/share/nautilus/python-extensions/tildr.py
nautilus -q
```

### Dolphin

Integration with Dolphin (KDE) is done via **KIO Service Menus**,
a `.desktop` file mechanism that adds custom actions to the context menu.
No additional dependencies are required beyond Dolphin itself.

**Features:**

* Context menu with submenu **Tildr → Add / Unlink / Restore**
* Supports single and multiple file selection
* Available for files and symlinks

**Manual installation:**

```sh
mkdir -p ~/.local/share/kio/servicemenus
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/dolphin/tildr.desktop \
  -o ~/.local/share/kio/servicemenus/tildr.desktop
```

> No restart required — Dolphin picks up new service menus automatically.

### Automatic installation

Both plugins are installed automatically by the Tildr installer script
when the respective file manager is detected on the system:

```sh
curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh
```

If the file manager is not installed at the time of Tildr installation,
you can install the plugin manually at any time using the commands above.

---

## Command Reference

> [!TIP]
> The `add`, `restore`, and `unlink` commands accept multiple file parameters simultaneously.
>
> ```sh
> tildr add .config/nvim/init.vim .config/nvim/lua/plugins.lua
> tildr restore .config/nvim/init.vim .config/nvim/lua/plugins.lua
> tildr unlink .config/nvim/init.vim .config/nvim/lua/plugins.lua
> ```
>
> If a path contains spaces, use single or double quotes:
>
> ```sh
> tildr unlink '.config/my app/config.toml' .config/nvim/init.vim
> ```

---

### `tildr init`

Initializes the Tildr repository and writes the user configuration file.

```sh
tildr init
tildr init --repo ~/.dotfiles-work
tildr init --no-git
tildr init --force
tildr init --quiet
```

Options:

| Flag            | Short| Description                                |
|-----------------|------|--------------------------------------------|
| `--repo <PATH>` | `-r` | Repository path, must be inside `$HOME`    |
| `--no-git`      |      | Skip `git init`                            |
| `--force`       | `-f` | Reinitialize even if config already exists |
| `--quiet`       | `-q` | Suppress output                            |

Behavior:

* Creates the repository directory
* Writes `~/.config/tildr/config.toml`
* Detects whether Git is available in `PATH`
* Writes `[git].available` to the config
* Initializes a Git repository only when Git is available and `--no-git` is not used
* Refuses `$HOME` itself as the repository root
* Refuses repositories outside `$HOME`
* If already initialized and `--force` is not passed, refreshes the saved Git availability state

---

### `tildr import`

Clone a remote dotfiles repository and apply it.

```sh
tildr import https://github.com/user/dotfiles
tildr import https://github.com/user/dotfiles ~/.dotfiles
tildr import https://github.com/user/dotfiles --force
```

Options:

| Flag              | Short | Description                                                        |
|-------------------|-------|--------------------------------------------------------------------|
| `--dest <PATH>`   | `-d`  | Local destination path (default: `~/.dotfiles`)                    |
| `--force`         | `-f`  | Overwrite existing config if it points to a different repo         |
| `--dry-run`       | `-n`  | Show what would be done without making changes                     |
| `--quiet`         | `-q`  | Suppress output                                                    |

Behavior:

* Clones the Git repository to the specified destination (or `~/.dotfiles`)
* Creates or updates `~/.config/tildr/config.toml` with the repository path
* Automatically runs `apply` to establish symlinks
* Destination must be inside `$HOME`
* Refuses to overwrite a config pointing to a different repository unless `--force` is used

---

### `tildr add`

Adds a file or a directory tree from `$HOME` into the repository and replaces each file with a symlink.

```sh
tildr add .bashrc
tildr add .config/nvim
tildr add .config/nvim/init.lua --dry-run
tildr add .bashrc .config/nvim/init.lua
tildr add .config/app01.json --nolink
```

Options:

| Flag        | Short | Description                                                           |
|-------------|-------|-----------------------------------------------------------------------|
| `--dry-run` | `-n`  | Preview actions without modifying files                               |
| `--nolink`  |       | Add to repository without creating a symlink (adds to `.tildrignore`) |
| `--force`   | `-f`  | Remove an existing repository target before moving the file           |
| `--quiet`   | `-q`  | Suppress output                                                       |

Behavior:

* Files are moved into the repository
* Symlinks are created at the original home path
* With `--nolink`, files are moved but no symlink is created and the file is added to `.tildrignore`
* Directories are traversed recursively
* Paths matched by `.tildrignore` are skipped
* Already-correct symlinks are skipped silently
* If `git.auto_commit = true`, the repository is auto-committed after a successful run

---

### `tildr apply`

Applies repository state into `$HOME` by creating or repairing symlinks.

```sh
tildr apply
tildr apply --dry-run
tildr apply --force --verbose
```

Options:

| Flag        | Short | Description                                                |
|-------------|-------|------------------------------------------------------------|
| `--dry-run` | `-n`  | Preview actions                                            |
| `--verbose` | `-v`  | Include unchanged or skipped entries in output             |
| `--force`   | `-f`  | Replace conflicting regular files or directories in `$HOME`|
| `--quiet`   | `-q`  | Suppress output                                            |

Behavior:

* Creates missing symlinks for all managed files
* Repairs broken or incorrect symlinks automatically
* Skips regular files already present in `$HOME` unless `--force` is used
* Uses the repository as the source of truth
* Does not modify repository content

---

### `tildr status`

Shows the synchronization state of all managed files.

```sh
tildr status
tildr status --json
tildr status --counter
```

Options:

| Flag        | Short| Description                    |
|-------------|------|--------------------------------|
| `--json`    | `-j` | Emit structured JSON output    |
| `--counter` | `-c` | Print aggregated counters only |

Status values:

| Status           | Meaning                                                        |
|------------------|----------------------------------------------------------------|
| `linked`         | Symlink exists and points to the correct repository file       |
| `missing_link`   | Repository file exists but the home symlink is absent          |
| `broken_symlink` | A symlink exists but points to a wrong target                  |
| `not_a_symlink`  | A regular file or directory exists where the symlink should be |

The `--counter` mode prints a summary like:

```text
Managed: 12
Linked: 10
Missing: 1
Broken: 0
Not symlink: 1
```

---

### `tildr list`

Lists managed files in the repository.

```sh
tildr list
tildr list --long
tildr list --tree
tildr list --export ~/tildr-files.json
tildr list --import ~/tildr-files.json
```

Options:

| Flag          | Short| Description                                                   |
|---------------|------|---------------------------------------------------------------|
| `--tree`      | `-t` | Show the repository as a directory tree                       |
| `--long`      | `-l` | Show type and file size for each entry                        |
| `--export`    |      | Export managed file list to a JSON file                       |
| `--import`    |      | Import managed file list from a JSON file and create symlinks |

Export JSON format:

```json
{
  "version": 1,
  "files": [
    ".bashrc",
    ".config/starship.toml",
    ".config/nvim/init.lua"
  ]
}
```

Notes:

* Standard listing includes only managed files discovered by repository scanning
* Tree view prints the repository directory structure directly
* `.tildrignore` patterns and internally excluded files are not shown
* Export creates a portable snapshot of managed files for use on other machines
* Import creates parent directories in `$HOME` as needed and skips files already correctly linked
* Files not found in the repository during import are reported as warnings but do not stop the process

---

### `tildr repo path`

Prints the absolute path to the configured Tildr repository.

```sh
tildr repo path
```

Useful for shell automation and aliases:

```sh
alias tildr-repo='cd "$(tildr repo path)"'
```

After that, `tildr-repo` takes you directly to the repository.

---

### `tildr git status`

Runs `git status` scoped to the Tildr repository.

```sh
tildr git status
```

Behavior:

* Uses the configured repository as both `--git-dir` and `--work-tree`
* Prints a success message when the repository is clean
* Prints an informational message plus the normal `git status` output when there are tracked or untracked changes

---

### `tildr cat`

Prints the content of a managed file from the repository.

```sh
tildr cat .bashrc
tildr cat .config/nvim/init.lua --less
tildr cat config
tildr cat
```

Options:

| Flag     | Short| Description            |
|----------|------|------------------------|
| `--less` | `-l` | Open output in a pager |

Behavior:

* If no target is passed, Tildr opens an interactive picker
* The special target `config` resolves to the Tildr config file itself
* When `--less` is used, Tildr respects `$PAGER` and falls back to `less -RFX` on TTY output

You can use your preferred file viewers by using the PAGER environment variable like this:

```sh
PAGER=bat tildr cat --less
```

---

### `tildr edit`

Opens a managed file in the repository with the configured editor.

```sh
tildr edit .bashrc
tildr edit
```

Behavior:

* If no target is passed, Tildr opens an interactive picker
* Editor resolution order: `$EDITOR` → `$VISUAL` → `nano`
* Edits are made directly in the repository file; the symlink in `$HOME` reflects the change immediately

You can open it with your preferred editor using the `EDITOR` environment variable like this:

```sh
EDITOR=vim tildr edit .bashrc
```

---

### `tildr unlink`

Removes symlinks from `$HOME` without touching the repository content.

```sh
tildr unlink .bashrc
tildr unlink .config/nvim
tildr unlink .bashrc .config/nvim/init.lua
tildr unlink --all
```

Options:

| Flag        |Short | Description               |
|-------------|------|---------------------------|
| `--all`     | `-a` | Unlink all managed files  |
| `--dry-run` | `-n` | Preview changes           |
| `--force`   | `-f` | Skip confirmation prompts |
| `--quiet`   | `-q` | Suppress output           |

Behavior:

* Only symlinks in `$HOME` are removed
* Repository files remain untouched
* Directory targets are expanded recursively over managed files
* Confirms before acting on a directory target unless `--force` is passed
* Empty parent directories in `$HOME` are cleaned up when possible
* If no target is passed, Tildr opens an interactive picker

---

### `tildr restore`

Moves managed files back from the repository into `$HOME` and removes the symlinks.

```sh
tildr restore .bashrc
tildr restore .config/nvim
tildr restore .bashrc .config/nvim/init.lua
tildr restore --all
```

Options:

| Flag        | Short| Description               |
|-------------|------|---------------------------|
| `--all`     | `-a` | Restore all managed files |
| `--dry-run` | `-n` | Preview changes           |
| `--force`   | `-f` | Skip confirmation prompts |
| `--quiet`   | `-q` | Suppress output           |

Behavior:

* Removes the home symlink if present
* Moves the real file back from the repository into `$HOME`
* Removes empty directories from the repository after restoration
* Confirms before acting on a directory target unless `--force` is passed
* If no target is passed, Tildr opens an interactive picker
* Auto-commits the repository when `git.auto_commit = true`

---

### `tildr del`

Deletes managed files from the repository and unlinks them from `$HOME`.

```sh
tildr del .bashrc
tildr del .config/nvim
tildr del --all
tildr del .bashrc --purge
```

Options:

| Flag        |Short | Description                                   |
|-------------|------|-----------------------------------------------|
| `--all`     | `-a` | Delete all managed files                      |
| `--dry-run` | `-n` | Preview changes                               |
| `--force`   | `-f` | Skip confirmation prompts                     |
| `--purge`   | `-p` | Permanently delete instead of moving to trash |
| `--quiet`   | `-q` | Suppress output                               |

Behavior:

* Removes the symlink from `$HOME` if it exists
* Deletes the file from the repository
* Default mode tries to send files to the system trash
* `--purge` permanently removes repository files without trash
* Confirms before acting on a directory target unless `--force` is passed
* If no target is passed, Tildr opens an interactive picker
* Auto-commits the repository when `git.auto_commit = true`

---

### `tildr mv`

Renames or moves a managed file inside the repository and updates its symlink in `$HOME`. Mirrors the behavior of the Linux `mv` command — rename and move are the same operation.

```sh
tildr mv .bashrc .bashrc_backup
tildr mv files/file.txt configs/file.txt
tildr mv
```

Options:

| Flag        | Short | Description                                    |
|-------------|-------|------------------------------------------------|
| `--dry-run` | `-n`  | Show what would be done without making changes |
| `--quiet`   | `-q`  | Suppress output                                |

Behavior:

* Renames or moves the file inside the repository
* Removes the old symlink from `$HOME`
* Creates a new symlink at the new path in `$HOME`
* If the destination is a filename only (no directory), the original directory is preserved
* If no arguments are passed, Tildr opens an interactive picker to select the source, then prompts for the new path
* Auto-commits the repository when `git.auto_commit = true`

---

### `tildr exclude`

Manages patterns in `.tildrignore` without editing the file manually.

```sh
tildr exclude add *.log
tildr exclude add cache/
tildr exclude remove *.log
tildr exclude list
```

Options:

| Subcommand        | Description                                              |
|-------------------|----------------------------------------------------------|
| `add <PATTERN>`   | Add a gitignore-style pattern to `.tildrignore`          |
| `remove <PATTERN>`| Remove the exact pattern from `.tildrignore`             |
| `list`            | Print all patterns in `.tildrignore`                     |

Behavior:

* Creates `.tildrignore` if it does not exist
* Duplicate patterns are ignored when adding
* Fails if the pattern is not found when removing
* Patterns use gitignore-style matching semantics
* Patterns added here prevent files from being discovered by `list`, `status`, and `apply`
* Does not remove existing symlinks — use `tildr unlink` for that

---

### `tildr open`

Opens the Tildr repository in the system file manager.

```sh
tildr open
```

Behavior:

* Launches the default file manager at the repository path
* Uses the `open` crate for cross-platform support (xdg-open on Linux, open on macOS)

---

### `tildr stats`

Shows statistics about managed files.

```sh
tildr stats
```

Output example:

```text
Managed files: 47
Total size:    2.3 MB
Largest:       .config/nvim/init.lua (12.4 KB)
By extension:  .toml (12), .lua (8), .sh (6), .json (5)
```

Behavior:

* Counts total managed files
* Calculates total size of managed files in `$HOME`
* Shows the largest managed file
* Shows file extension distribution (top 6)

---

### `tildr backup`

Creates a compressed tarball backup of the repository.

```sh
tildr backup
tildr backup --output ~/my-backup.tar.gz
```

Options:

| Flag              | Description                                |
|-------------------|--------------------------------------------|
| `--output <FILE>` | Custom output path for the backup file     |

Behavior:

* Creates a `.tar.gz` archive of the entire repository
* Default output: `~/.dotfiles-backup-YYYY-MM-DD.tar.gz`
* Shows the backup file size after creation
* Requires `tar` to be installed on the system

---

### `tildr suggest`

Scans `$HOME` for common dotfile patterns that could be managed by Tildr.

```sh
tildr suggest
```

Output example:

```text
Suggested files in $HOME:

  Shell:        .zshrc, .bash_profile
  Editor:       .config/nvim/init.lua
  Terminal:     .config/alacritty.toml
  Git:          .gitconfig
  Tools:        .tmux.conf

Run `tildr add <file>` to manage them.
```

Behavior:

* Checks for common dotfile patterns (shell configs, editor configs, terminal emulators, git, etc.)
* Skips files already managed by Tildr
* Reports suggestions grouped by category
* Does not modify any files

---

### `tildr group`

Manages named groups of managed files for batch operations.

```sh
tildr group list
tildr group create dev --files .bashrc .config/nvim
tildr group add dev --files .tmux.conf
tildr group remove dev --files .tmux.conf
tildr group delete dev
tildr group apply dev
tildr group unlink dev
```

Options:

| Subcommand                        | Description                                     |
|-----------------------------------|-------------------------------------------------|
| `create <NAME> --files <FILES>`   | Create a new group with the specified files     |
| `add <NAME> --files <FILES>`      | Add files to an existing group                  |
| `remove <NAME> --files <FILES>`   | Remove files from a group                       |
| `delete <NAME>`                   | Delete a group                                  |
| `list`                            | List all groups and their files                 |
| `apply <NAME>`                    | Create symlinks for all files in the group      |
| `unlink <NAME>`                   | Remove symlinks for all files in the group      |

Behavior:

* Groups are stored in `.tildr/groups.json` in the repository root
* `apply` creates symlinks in `$HOME` for all files in the group
* `unlink` removes symlinks from `$HOME` for all files in the group
* Group operations work on files already managed by Tildr

---

### `tildr secret`

Manages encryption of sensitive files in your dotfiles repository using GPG encryption.

Some files you manage with Tildr — such as SSH keys, GPG private keys, or any file containing credentials — should never be stored in plain text in a repository, especially a public one. `tildr secret` solves this by encrypting those files into a single encrypted bundle that is safe to commit and push.

#### How it works

Tildr maintains two files at the root of your repository:

* `.tildr/encrypted-items` — a plaintext manifest listing the relative paths of all registered sensitive files, one per line. This file is committed to the repository.
* `.tildr/encrypted.gpg` — an encrypted bundle containing all the registered files packed together. This file is also committed to the repository.

The original sensitive files are **never stored in the repository**. When you register a file with `tildr secret add`, Tildr automatically adds it to `.gitignore` and removes it from Git tracking if it was already being tracked. Only the encrypted bundle enters version control.

#### Encryption model

Tildr supports two GPG encryption modes, configured via `[crypto].mode` in `config.toml`:

**Symmetric** (`mode = "symmetric"`, default):

* No key pair required — only a passphrase
* GPG prompts for the passphrase via the system pinentry on first use
* The same passphrase must be used to decrypt on any machine
* Simpler setup, suitable for single-user environments

**Asymmetric** (`mode = "asymmetric"`):

* Uses an existing GPG key pair — no separate passphrase to remember
* `[crypto].gpg_key` must be set to the recipient key ID or email, or Tildr prompts interactively on first use and saves the choice automatically
* Decryption uses the private key silently (subject to GPG Agent caching)
* Preferred when you already manage GPG keys and want a seamless new-machine setup

In both modes:

* GPG must be installed on the system (`gpg` in `PATH`)
* Tildr creates a tar archive of all registered files and encrypts it as a single `.tildr/encrypted.gpg` bundle
* Files are archived with **relative paths** so they extract correctly to any `$HOME` regardless of username or machine
* Decryption is always automatic — GPG detects the encryption type from the bundle

#### Subcommands

```sh
tildr secret add <FILE>
tildr secret remove <FILE>
tildr secret list
tildr secret encrypt
tildr secret decrypt
```

---

#### `tildr secret add`

Registers a sensitive file, adds it to `.gitignore`, removes it from Git tracking if necessary, and re-encrypts the full bundle.

```sh
tildr secret add ~/.ssh/id_rsa
tildr secret add ~/.gnupg/private-keys-v1.d/ABC123.key
```

Behavior:

* The file must exist in `$HOME`
* The relative path is added to `.tildr/encrypted-items`
* The file path is appended to `.gitignore` in the repository root so it is never committed
* If the file was already tracked by Git, `git rm --cached` is run to remove it from the index without deleting the file from disk
* All registered files (including the newly added one) are re-packed into a tar archive and re-encrypted into `.tildr/encrypted.gpg`
* GPG will prompt for a passphrase (symmetric) or use the configured key (asymmetric) on encryption
* Auto-commits the repository when `git.auto_commit = true`

> **Important:** the file registered with `tildr secret add` must be from `$HOME`, not from the repository. The original file lives in `$HOME` and only the encrypted bundle lives in the repository.

---

#### `tildr secret remove`

Unregisters a sensitive file from the manifest and re-encrypts the bundle without it.

```sh
tildr secret remove .ssh/id_rsa
```

Behavior:

* The relative path is removed from `.tildr/encrypted-items`
* If other files remain registered, the bundle is re-encrypted without the removed entry
* If no files remain, `.tildr/encrypted.gpg` is deleted from the repository
* The original file in `$HOME` is not touched
* Auto-commits the repository when `git.auto_commit = true`

---

#### `tildr secret list`

Lists all sensitive files currently registered in `.tildr/encrypted-items`.

```sh
tildr secret list
```

Output example:

```text
Sensitive files
---------------
  .ssh/id_rsa
  .ssh/id_rsa.pub
  .gnupg/private-keys-v1.d/ABC123.key
```

---

#### `tildr secret encrypt`

Manually re-encrypts all registered files into the bundle using their current content from `$HOME`.

```sh
tildr secret encrypt
```

Behavior:

* Reads all entries from `.tildr/encrypted-items`
* Packs the current content of each file from `$HOME` into a tar archive
* Encrypts the archive into `.tildr/encrypted.gpg`, replacing the previous bundle
* In symmetric mode, GPG may prompt for a passphrase depending on the agent cache state
* In asymmetric mode, GPG uses the configured key silently (subject to GPG Agent caching)
* Auto-commits the repository when `git.auto_commit = true`

Use this command after editing a registered sensitive file when you want to update the bundle manually. If you use `tildr sync`, re-encryption happens automatically before the push — so in a typical workflow, running `tildr secret encrypt` explicitly is optional.

---

#### `tildr secret decrypt`

Decrypts the bundle and restores all registered files to their original locations in `$HOME`.

```sh
tildr secret decrypt
```

Behavior:

* Decrypts `.tildr/encrypted.gpg` using GPG — the encryption type (symmetric or asymmetric) is detected automatically from the bundle
* Extracts the tar archive into `$HOME`, restoring each file to its registered path
* In symmetric mode, GPG prompts for the passphrase via the system pinentry
* In asymmetric mode, GPG uses the private key silently (subject to GPG Agent caching)
* Files are extracted with relative paths — they always land correctly regardless of username

Use this command when you need to restore sensitive files manually, for example after running `tildr secret remove` or after setting up a new machine without going through `tildr import`.

---

#### Integration with `tildr sync`

When `tildr sync` is about to push commits to the remote, it automatically re-encrypts all registered files before the push. This ensures the bundle in the remote repository always reflects the current state of your sensitive files, even if you edited them since the last encryption.

Re-encryption only happens on push scenarios (`PushOnly` and `Diverged`). Pull-only and up-to-date scenarios do not trigger re-encryption.

---

#### Integration with `tildr import`

When `tildr import` is run and the cloned repository contains a `.tildr/encrypted.gpg` bundle, Tildr automatically decrypts it after applying symlinks.

* In symmetric mode, GPG prompts for the passphrase via the system pinentry
* In asymmetric mode, GPG uses the private key — which must already be available in the keyring at import time (e.g. restored by `tildr apply` if you manage `~/.gnupg` with Tildr)

If GPG is not installed at import time, Tildr warns but does not fail. You can decrypt later manually with `tildr secret decrypt` once GPG is available.

---

#### GPG Agent and passphrase caching

GPG uses a background agent (`gpg-agent`) that caches credentials in memory for a period of time after the first successful use.

In **symmetric mode**, the agent caches the passphrase — after you enter it once, subsequent GPG operations including `tildr secret encrypt`, `tildr secret decrypt`, and the automatic re-encryption in `tildr sync` may not prompt for the passphrase again until the cache expires.

In **asymmetric mode**, the agent caches the private key passphrase if the key has one. Operations proceed silently while the cache is active.

Default cache duration is **600 seconds (10 minutes)**. You can configure it in `~/.gnupg/gpg-agent.conf`:

```text
default-cache-ttl 600
max-cache-ttl 7200
```

To force GPG to forget the cached passphrase immediately:

```sh
gpg-connect-agent reloadagent /bye
```

This behavior is managed entirely by GPG and the system pinentry — Tildr has no control over passphrase caching.

---

#### Inspecting the bundle

To list the files inside the bundle without decrypting them to disk:

```sh
gpg --decrypt .tildr/encrypted.gpg 2>/dev/null | tar tv
```

This is useful to verify the bundle contents are correct before pushing to a remote repository.

---

#### Typical secret management workflow

```sh
# Register sensitive files
tildr secret add ~/.ssh/id_rsa
tildr secret add ~/.ssh/id_rsa.pub

# Verify what is registered
tildr secret list

# Edit a registered file, then update the bundle
tildr secret encrypt

# Push everything including the updated bundle
tildr sync

# On a new machine after tildr import
tildr secret decrypt
```

---

### `tildr sync`

Synchronizes the repository with its configured Git remote in both directions.

```sh
tildr sync
tildr sync --dry-run
tildr sync --force
```

Options:

| Flag        |Short | Description                                           |
|-------------|------|-------------------------------------------------------|
| `--dry-run` | `-n` | Preview pull / merge / push actions without executing |
| `--force`   | `-f` | Pass `--force` to the final `git push`                |
| `--quiet`   | `-q` | Suppress output                                       |

Behavior:

* Uses the current branch dynamically
* Uses the branch tracking remote and upstream branch from Git config
* Fetches from the tracked remote before deciding what to do
* If only local commits exist, pushes them
* If only remote commits exist, performs a fast-forward pull
* If both local and remote commits exist, simulates a merge first
* Aborts safely and reports conflicting files when a merge conflict would occur
* Uses the saved Git availability from Tildr config instead of probing `PATH` on every run

---

### `tildr doctor`

Runs a health check against the Tildr environment and reports any issues found.

```sh
tildr doctor
```

Checks performed:

| Check       | What is verified                                                                   |
|-------------|------------------------------------------------------------------------------------|
| Repository  | Repository directory exists                                                        |
| Config      | Config file exists at expected path                                                |
| Git         | Repository is a Git repo and the working tree has no pending or untracked changes  |
| Permissions | Repository and managed files are accessible                                        |
| Disk        | Repository total size                                                              |
| Symlinks    | All managed symlinks are correct; reports broken or missing links                  |

Output example:

```text
Checking environment...

✓ Repository    OK
✓ Config        OK
✓ Git           OK
✓ Permissions   OK
✓ Disk          OK (42.3 KB)
✓ Symlinks      OK

All checks passed
```

---

### `tildr completions`

Generate shell completion scripts for various shells.

```sh
tildr completions bash
tildr completions zsh
tildr completions fish
```

Argument:

The shell to generate completions for: `bash`, `zsh`, or `fish`.

Installation:

**Bash:**

```sh
tildr completions bash >> ~/.bash_completion
```

**Zsh (Oh My Zsh):**

```sh
tildr completions zsh > ~/.oh-my-zsh/completions/_tildr
```

**Zsh (vanilla):**

Add the following to `~/.zshrc`:

```sh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

Then install the completions:

```sh
tildr completions zsh > ~/.zfunc/_tildr
```

Finally, restart your shell or run `exec zsh` for the changes to take effect.

**Fish:**

```sh
tildr completions fish > ~/.config/fish/completions/tildr.fish
```

Behavior:

* Generates and outputs shell-specific completion scripts
* No interactive installation — output is printed to stdout for you to redirect or pipe
* Each shell has its own completion script format and installation location

---

### `tildr info`

Displays project metadata.

```sh
tildr info credits
tildr info license
```

Modes:

| Mode      | Output                                                                     |
|-----------|----------------------------------------------------------------------------|
| `credits` | Build metadata, repository URL, license, commit hash, maintainer, homepage |
| `license` | Installed license text, opened through a pager when available              |

---

## Interactive Behavior

When a target is omitted, the following commands open an interactive file picker over the list of managed files:

* `tildr add` (picks from `$HOME` instead of the repository)
* `tildr cat`
* `tildr edit`
* `tildr unlink`
* `tildr restore`
* `tildr del`
* `tildr mv`

The picker operates on the managed files discovered by scanning the repository.

When the number of managed files exceeds `core.search_threshold` (default: `15`), the picker shows a search step first. Type a fragment to filter the list by fuzzy match, or press enter with an empty input to skip filtering and see the full list.

---

## Typical Workflow

Initial setup and daily use:

```sh
tildr init
tildr add .bashrc
tildr add .config/nvim
tildr git status
tildr status
tildr apply
tildr sync
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
# Symmetric mode (default) — uses a passphrase
tildr secret add ~/.ssh/id_rsa
tildr secret list
tildr secret encrypt
tildr sync

# Asymmetric mode — set in config.toml first
# [crypto]
# mode = "asymmetric"
# gpg_key = "william@email.com"
tildr secret add ~/.ssh/id_rsa
tildr sync
```

---

## Operational Notes

* Tildr is designed for home-directory management on Linux and macOS
* The repository must stay inside `$HOME`
* Relative paths for managed targets are interpreted from `$HOME`
* Directory operations are always recursive over files under that path
* `apply` does not overwrite conflicting regular files unless `--force` is provided
* `unlink` removes only symlinks, never repository content
* `restore` physically moves the real file back out of the repository
* `del` removes repository content; use `--purge` for permanent deletion, otherwise files go to trash
* `git.auto_commit` affects `add`, `restore`, `del`, `mv`, and `secret` — not `apply`, `unlink`, `git`, or `sync`
* `git.enable = false` disables Tildr-managed Git operations even if Git is installed
* `tildr secret` requires `gpg` to be installed and available in `PATH`
* sensitive files registered with `tildr secret add` are never stored in plain text in the repository
* `crypto.mode` controls whether symmetric (passphrase) or asymmetric (key pair) GPG encryption is used
* in asymmetric mode, `crypto.gpg_key` is saved automatically after interactive key selection on first use
* `core.color = false` disables all colored output; `NO_COLOR` environment variable is also respected

---

## Summary

Tildr turns `$HOME` into a repository-backed declarative environment. Its model is simple:

* store real files in a repository
* expose them into `$HOME` through symlinks
* inspect drift with `status`
* converge state with `apply`
* recover ownership with `restore`

For reliable operation, keep the repository in your home directory, use `.tildrignore` to exclude unmanaged artifacts, and treat `tildr repo path` as the canonical way to locate the repository in scripts and shell aliases.
