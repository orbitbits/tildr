---
title: TILDR-COMMANDS
section: 1
header: User Commands
footer: Tildr
date: 2026
---

# NAME

tildr-commands — command reference for Tildr

# SYNOPSIS

**tildr** *\<command\>* \[*options*\] \[*target*\]

# DESCRIPTION

This page documents all commands available in **tildr(1)**.

The **add**, **restore**, and **unlink** commands accept multiple file parameters simultaneously:

```sh
tildr add .config/nvim/init.vim .config/nvim/lua/plugins.lua
tildr restore .config/nvim/init.vim .config/nvim/lua/plugins.lua
```

If a path contains spaces, use single or double quotes:

```sh
tildr unlink '.config/my app/config.toml' .config/nvim/init.vim
```

# COMMANDS

## tildr init

Initializes the Tildr repository and writes the user configuration file.

```sh
tildr init
tildr init --repo ~/.dotfiles-work
tildr init --no-git
tildr init --force
```

**Options:**

**-r**, **--repo** *\<PATH\>*
:   Repository path, which must be inside `$HOME`.

**--no-git**
:   Skip `git init`.

**-f**, **--force**
:   Reinitialize even if config already exists.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Creates the repository directory
- Writes `~/.config/tildr/config.toml`
- Detects whether Git is available in `PATH`
- Writes `[git].available` to the config
- Initializes a Git repository only when Git is available and `--no-git` is not used
- Refuses `$HOME` itself as the repository root
- Refuses repositories outside `$HOME`
- If already initialized and `--force` is not passed, refreshes the saved Git availability state

## tildr import

Clone a remote dotfiles repository and apply it.

```sh
tildr import https://github.com/user/dotfiles
tildr import https://github.com/user/dotfiles ~/.dotfiles
tildr import https://github.com/user/dotfiles --force
```

**Options:**

**-d**, **--dest** *\<PATH\>*
:   Local destination path (default: `~/.dotfiles`).

**-f**, **--force**
:   Overwrite existing `config.toml` if it points to a different repo.

**-n**, **--dry-run**
:   Show what would be done without making changes.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Clones the Git repository to the specified destination (or `~/.dotfiles`)
- Creates or updates `~/.config/tildr/config.toml` with the repository path
- Runs `apply` automatically to establish symlinks
- Destination must be inside `$HOME`
- Refuses to overwrite a config pointing to a different repository unless `--force` is used

## tildr add

Adds a file or a directory tree from `$HOME` into the repository and replaces each file with a symlink.

```sh
tildr add .bashrc
tildr add .config/nvim
tildr add .config/nvim/init.lua --dry-run
tildr add .bashrc .config/nvim/init.lua
tildr add .config/app01.json --nolink
```

**Options:**

**-n**, **--dry-run**
:   Preview actions without modifying files.

**--nolink**
:   Add to repository without creating a symlink. The file is added to `.tildrignore` automatically.

**-f**, **--force**
:   Remove an existing repository target before moving the file.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- If no path is passed, Tildr opens an interactive picker to select a file from `$HOME`
- Files are moved into the repository
- Symlinks are created at the original home path
- With `--nolink`, files are moved but no symlink is created and the file is added to `.tildrignore`
- Directories are traversed recursively
- Paths matched by `.tildrignore` are skipped
- Already-correct symlinks are skipped silently
- If `git.auto_commit = true`, the repository is auto-committed after a successful run

## tildr apply

Applies repository state into `$HOME` by creating or repairing symlinks.

```sh
tildr apply
tildr apply --dry-run
tildr apply --force --verbose
```

**Options:**

**-n**, **--dry-run**
:   Preview actions.

**-v**, **--verbose**
:   Include unchanged or skipped entries in output.

**-f**, **--force**
:   Replace conflicting regular files or directories in `$HOME`.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Creates missing symlinks for all managed files
- Repairs broken or incorrect symlinks automatically
- Skips regular files already present in `$HOME` unless `--force` is used
- Uses the repository as the source of truth
- Does not modify repository content

## tildr status

Shows the synchronization state of all managed files.

```sh
tildr status
tildr status --json
tildr status --counter
```

**Options:**

**-j**, **--json**
:   Emit structured JSON output.

**-c**, **--counter**
:   Print aggregated counters only.

**Status values:**

`linked`
:   Symlink exists and points to the correct repository file.

`missing_link`
:   Repository file exists but the home symlink is absent.

`broken_symlink`
:   A symlink exists but points to a wrong target.

`not_a_symlink`
:   A regular file or directory exists where the symlink should be.

The **--counter** mode prints a summary:

```text
Managed: 12
Linked: 10
Missing: 1
Broken: 0
Not symlink: 1
```

## tildr list

Lists managed files in the repository.

```sh
tildr list
tildr list --long
tildr list --tree
tildr list --export ~/tildr-files.json
tildr list --import ~/tildr-files.json
```

**Options:**

**-t**, **--tree**
:   Show the repository as a directory tree.

**-l**, **--long**
:   Show type and file size for each entry.

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

**Notes:**

- Standard listing includes only managed files discovered by repository scanning
- Tree view prints the repository directory structure directly
- `.tildrignore` patterns and internally excluded files are not shown
- Export creates a portable snapshot of managed files for use on other machines
- Import creates parent directories in `$HOME` as needed and skips files already correctly linked

## tildr repo path

Prints the absolute path to the configured Tildr repository.

```sh
tildr repo path
```

Useful for shell automation and aliases:

```sh
alias tildr-repo='cd "$(tildr repo path)"'
```

## tildr git status

Runs `git status` scoped to the Tildr repository.

```sh
tildr git status
```

**Behavior:**

- Uses the configured repository as both `--git-dir` and `--work-tree`
- Prints a success message when the repository is clean
- Prints an informational message plus the normal `git status` output when there are tracked or untracked changes

## tildr cat

Prints the content of a managed file from the repository.

```sh
tildr cat .bashrc
tildr cat .config/nvim/init.lua --less
tildr cat config
tildr cat
```

You can use your preferred file viewers by using the PAGER environment variable like this:

```sh
PAGER=bat tildr cat --less
```

**Options:**

**-l**, **--less**
:   Open output in a pager.

**Behavior:**

- If no target is passed, Tildr opens an interactive picker
- The special target `config` resolves to the Tildr config file itself
- When `--less` is used, Tildr respects `$PAGER` and falls back to `less -RFX` on TTY output

## tildr edit

Opens a managed file in the repository with the configured editor.

```sh
tildr edit .bashrc
tildr edit
```

You can open it with your preferred editor using the `EDITOR` environment variable like this:

```sh
EDITOR=vim tildr edit .bashrc
```

**Behavior:**

- If no target is passed, Tildr opens an interactive picker
- Editor resolution order: `$EDITOR`, then `$VISUAL`, then `nano`
- Edits are made directly in the repository file; the symlink in `$HOME` reflects the change immediately

## tildr unlink

Removes symlinks from `$HOME` without touching the repository content.

```sh
tildr unlink .bashrc
tildr unlink .config/nvim
tildr unlink .bashrc .config/nvim/init.lua
tildr unlink --all
```

**Options:**

**-a**, **--all**
:   Unlink all managed files.

**-n**, **--dry-run**
:   Preview changes.

**-f**, **--force**
:   Skip confirmation prompts.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Only symlinks in `$HOME` are removed
- Repository files remain untouched
- Directory targets are expanded recursively over managed files
- Confirms before acting on a directory target unless `--force` is passed
- Empty parent directories in `$HOME` are cleaned up when possible
- If no target is passed, Tildr opens an interactive picker

## tildr restore

Moves managed files back from the repository into `$HOME` and removes the symlinks.

```sh
tildr restore .bashrc
tildr restore .config/nvim
tildr restore .bashrc .config/nvim/init.lua
tildr restore --all
```

**Options:**

**-a**, **--all**
:   Restore all managed files.

**-n**, **--dry-run**
:   Preview changes.

**-f**, **--force**
:   Skip confirmation prompts.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Removes the home symlink if present
- Moves the real file back from the repository into `$HOME`
- Removes empty directories from the repository after restoration
- Confirms before acting on a directory target unless `--force` is passed
- If no target is passed, Tildr opens an interactive picker
- Auto-commits the repository when `git.auto_commit = true`

## tildr del

Deletes managed files from the repository and unlinks them from `$HOME`.

```sh
tildr del .bashrc
tildr del .config/nvim
tildr del --all
tildr del .bashrc --purge
```

**Options:**

**-a**, **--all**
:   Delete all managed files.

**-n**, **--dry-run**
:   Preview changes.

**-f**, **--force**
:   Skip confirmation prompts.

**-p**, **--purge**
:   Permanently delete instead of moving to trash.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Removes the symlink from `$HOME` if it exists
- Deletes the file from the repository
- Default mode tries to send files to the system trash
- `--purge` permanently removes repository files without trash
- Confirms before acting on a directory target unless `--force` is passed
- If no target is passed, Tildr opens an interactive picker
- Auto-commits the repository when `git.auto_commit = true`

## tildr mv

Renames or moves a managed file inside the repository and updates its symlink in `$HOME`. Mirrors the behavior of the Linux `mv` command — rename and move are the same operation.

```sh
tildr mv .bashrc .bashrc_backup
tildr mv files/file.txt configs/file.txt
tildr mv
```

**Options:**

**-n**, **--dry-run**
:   Show what would be done without making changes.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Renames or moves the file inside the repository
- Removes the old symlink from `$HOME`
- Creates a new symlink at the new path in `$HOME`
- If the destination is a filename only (no directory), the original directory is preserved
- If no arguments are passed, Tildr opens an interactive picker to select the source, then prompts for the new path
- Auto-commits the repository when `git.auto_commit = true`

## tildr exclude

Manages patterns in `.tildrignore` without editing the file manually.

```sh
tildr exclude add *.log
tildr exclude add cache/
tildr exclude remove *.log
tildr exclude list
```

**Subcommands:**

**add** *\<PATTERN\>*
:   Adds a gitignore-style pattern to `.tildrignore`. Duplicate patterns are ignored.

**remove** *\<PATTERN\>*
:   Removes the exact pattern from `.tildrignore`. Fails if the pattern is not found.

**list**
:   Prints all non-empty, non-comment lines from `.tildrignore`.

**Behavior:**

- Creates `.tildrignore` if it does not exist
- Patterns use gitignore-style matching semantics
- Patterns added here prevent files from being discovered by `list`, `status`, and `apply`
- Does not remove existing symlinks — use `tildr unlink` for that

## tildr open

Opens the Tildr repository in the system file manager.

```sh
tildr open
```

**Behavior:**

- Launches the default file manager of the system at the repository path
- Uses the `open` crate for cross-platform support (xdg-open on Linux, open on macOS)

## tildr stats

Shows statistics about managed files.

```sh
tildr stats
```

**Output example:**

```text
Managed files: 47
Total size:    2.3 MB
Largest:       .config/nvim/init.lua (12.4 KB)
By extension:  .toml (12), .lua (8), .sh (6), .json (5)
```

**Behavior:**

- Counts total managed files
- Calculates total size of managed files in `$HOME`
- Shows the largest managed file
- Shows file extension distribution (top 6)

## tildr backup

Creates a compressed tarball backup of the repository.

```sh
tildr backup
tildr backup --output ~/my-backup.tar.gz
```

**Options:**

**--output** *\<FILE\>*
:   Custom output path for the backup file.

**Behavior:**

- Creates a `.tar.gz` archive of the entire repository
- Default output: `~/.dotfiles-backup-YYYY-MM-DD.tar.gz`
- Shows the backup file size after creation
- Requires `tar` to be installed on the system

## tildr secret

Manages encryption of sensitive files in your dotfiles repository using GPG.

```sh
tildr secret add ~/.ssh/id_rsa
tildr secret remove .ssh/id_rsa
tildr secret list
tildr secret encrypt
tildr secret decrypt
```

**Subcommands:**

**add** *\<FILE\>*
:   Registers a file as sensitive, adds it to `.gitignore`, removes it from Git tracking if already tracked, and re-encrypts the full bundle.

**remove** *\<FILE\>*
:   Unregisters a file from the manifest and re-encrypts the bundle without it. If no files remain, the bundle is deleted. The original file in `$HOME` is not touched.

**list**
:   Lists all sensitive files currently registered in `.tildr-encrypt`.

**encrypt**
:   Manually re-encrypts all registered files into the bundle using their current content from `$HOME`.

**decrypt**
:   Decrypts the bundle and restores all registered files to their original locations in `$HOME`.

**Behavior:**

- GPG must be installed and available in `PATH`
- Two encryption modes are supported, configured via `[crypto].mode` in `config.toml`:
  - `symmetric` (default) — passphrase only, no key pair required
  - `asymmetric` — uses an existing GPG key pair; `[crypto].gpg_key` sets the recipient
- In asymmetric mode, if `gpg_key` is not set, Tildr prompts interactively and saves the chosen key to config
- The original sensitive files are never stored in plain text in the repository
- Only the encrypted bundle `.tildr-encrypt.gpg` and the plaintext manifest `.tildr-encrypt` are committed
- `tildr sync` automatically re-encrypts before pushing
- `tildr import` automatically decrypts after cloning if a bundle is present
- Auto-commits the repository when `git.auto_commit = true`

## tildr sync

Synchronizes the repository with its configured Git remote in both directions.

```sh
tildr sync
tildr sync --dry-run
tildr sync --force
```

**Options:**

**-n**, **--dry-run**
:   Preview pull, merge, and push actions without executing.

**-f**, **--force**
:   Pass `--force` to the final `git push`.

**-q**, **--quiet**
:   Suppress output.

**Behavior:**

- Uses the current branch dynamically
- Uses the branch tracking remote and upstream branch from Git config
- Fetches from the tracked remote before deciding what to do
- If only local commits exist, pushes them
- If only remote commits exist, performs a fast-forward pull
- If both local and remote commits exist, simulates a merge first
- Aborts safely and reports conflicting files when a merge conflict would occur
- Uses the saved Git availability from Tildr config instead of probing `PATH` on every run

## tildr doctor

Runs a health check against the Tildr environment and reports any issues.

```sh
tildr doctor
```

**Checks performed:**

Repository
:   Repository directory exists.

Config
:   Config file exists at expected path.

Git
:   Repository is a Git repo and the working tree has no pending or untracked changes.

Permissions
:   Repository and managed files are accessible.

Disk
:   Repository total size on disk.

Symlinks
:   All managed symlinks are correct; reports broken or missing links.

## tildr completions

Generate shell completion scripts for various shells.

```sh
tildr completions bash
tildr completions zsh
tildr completions fish
```

**Argument:**

The shell to generate completions for: `bash`, `zsh`, or `fish`.

**Installation:**

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

Then install completions:

```sh
tildr completions zsh > ~/.zfunc/_tildr
```

Finally, restart your shell or run `exec zsh` for changes to take effect.

**Fish:**

```sh
tildr completions fish > ~/.config/fish/completions/tildr.fish
```

**Behavior:**

- Generates and outputs shell-specific completion scripts
- No interactive installation — output is printed to stdout for you to redirect or pipe
- Each shell has its own completion script format and installation location

## tildr info

Displays project metadata.

```sh
tildr info credits
tildr info license
```

**Modes:**

`credits`
:   Build metadata, repository URL, license, commit hash, maintainer, homepage.

`license`
:   Installed license text, opened through a pager when available.

# INTERACTIVE BEHAVIOR

When a target is omitted, the following commands open an interactive file picker over the list of managed files:

- `add` (picks from `$HOME` instead of the repository)
- `cat`
- `edit`
- `unlink`
- `restore`
- `del`
- `mv`

When the number of managed files exceeds `core.search_threshold` (default: `15`), the picker shows a search step first. Type a fragment to filter the list by fuzzy match, or press enter with an empty input to skip filtering and see the full list.

# TYPICAL WORKFLOW

Initial setup:

```sh
tildr init
tildr add .bashrc
tildr add .config/nvim
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
tildr secret add ~/.ssh/id_rsa
tildr secret list
tildr secret encrypt
tildr sync
```

# OPERATIONAL NOTES

- Tildr is designed for home-directory management on Linux and macOS
- The repository must stay inside `$HOME`
- Relative paths for managed targets are interpreted from `$HOME`
- Directory operations are recursive over all files under that path
- `apply` does not overwrite conflicting regular files unless `--force` is provided
- `unlink` removes only symlinks, never repository content
- `restore` physically moves the real file back out of the repository
- `del` removes repository content; use `--purge` for permanent deletion
- `git.auto_commit` affects `add`, `restore`, `del`, `mv`, and `secret` — not `apply`, `unlink`, `git`, or `sync`
- `tildr secret` requires `gpg` to be installed and available in `PATH`
- sensitive files registered with `tildr secret add` are never stored in plain text in the repository
- `core.color = false` in `config.toml` disables all colored output; the `NO_COLOR` environment variable is also respected

# SEE ALSO

**tildr(1)**, **tildr-config(1)**, **tildr-security(1)**, **tildr-plugins(1)**
