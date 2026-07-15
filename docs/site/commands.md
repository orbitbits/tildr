---
layout: doc
part: 7
section: CLI Reference
menu: tildr
version: "0.2.0"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Core Commands
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.2.0/commands/
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
tildr status --less
```

Options:

| Flag        | Short| Description                                            |
|-------------|------|--------------------------------------------------------|
| `--json`    | `-j` | Emit structured JSON output                            |
| `--counter` | `-c` | Print aggregated counters only                         |
| `--less`    | `-l` | View the output in an interactive pager (`less -RFX`)  |

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
tildr list --less
tildr list --export ~/tildr-files.json
tildr list --import ~/tildr-files.json
```

**Options:**

**-t**, **--tree**
:   Show the repository as a directory tree.

**-l**, **--long**
:   Show type and file size for each entry.

**-l**, **--less**
:   View the output in an interactive pager (uses `$PAGER` or `less -RFX`).

**--export** *\<FILE\>*
:   Export the list of managed files to a JSON file. The JSON contains a version number, export timestamp, and an array of relative file paths.

**--import** *\<FILE\>*
:   Import a previously exported JSON file and create symlinks for all listed files in `$HOME`. Files already correctly linked are skipped. Files not found in the repository are reported as warnings.

**Export JSON format:**

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

### `tildr snapshot`

Generate a reproducible bootstrap script from the current Tildr setup.

```sh
tildr snapshot > setup.sh
tildr snapshot --output ~/setup.sh
chmod +x setup.sh
./setup.sh
```

**Options:**

**--output** *\<FILE\>*
:   Custom output file path. If omitted, prints to stdout.

**Behavior:**

* Generates a shell script that reproduces the entire Tildr setup on a new machine
* Auto-detects the git remote URL for the clone step
* Checks prerequisites (git, tildr, optionally gpg)
* Clones the repository or creates it if it doesn't exist
* Initializes Tildr config
* Runs `tildr apply` to create symlinks
* Decrypts secrets if `.tildr/encrypted.gpg` exists
* Output is idempotent — safe to run multiple times

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
