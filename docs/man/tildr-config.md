---
title: TILDR-CONFIG
section: 1
header: User Commands
footer: Tildr
date: 2026
---

# NAME

tildr-config — configuration reference for Tildr

# SYNOPSIS

`~/.config/tildr/config.toml`

# DESCRIPTION

Tildr stores its user configuration in TOML format at `~/.config/tildr/config.toml`.

If the XDG config directory is unavailable, Tildr falls back to `$HOME/.config/tildr/config.toml`.

The config file is created by **tildr init** and is never written automatically outside of that command.
If the file does not exist at startup, all default values are applied silently.

# CONFIGURATION SECTIONS

## [core]

The `[core]` section controls general Tildr behavior.

**core.repo**
:   Repository path used by the CLI. Accepts `~/...` notation or an absolute path inside `$HOME`.
    The repository must be inside `$HOME` and cannot be `$HOME` itself.
    Default: `~/.dotfiles`

    Example:
    ```toml
    [core]
    repo = "~/.dotfiles"
    ```

**core.search_threshold**
:   Number of managed files above which interactive pickers show a search/filter step before
    the selection list. Type a fragment to filter by fuzzy match, or press enter to skip.
    Default: `15`

    Example:
    ```toml
    [core]
    search_threshold = 20
    ```

**core.color**
:   When `false`, disables all colored output by setting `NO_COLOR=1` before dispatch.
    The `NO_COLOR` environment variable is also respected regardless of this setting.
    Default: `true`

    Example:
    ```toml
    [core]
    color = false
    ```

## [git]

The `[git]` section controls Git integration.

**git.available**
:   Whether Git was detected by **tildr init**. Written automatically by Tildr and used by
    Git-aware commands such as **sync**, **git status**, and auto-commit flows.
    Default when no config exists: `true`

    This value is updated automatically when you run `tildr init`.

**git.enable**
:   Optional manual override. When explicitly set to `false`, Tildr skips Git operations even if
    Git is installed and `git.available = true`.
    Default: unset

    Example:
    ```toml
    [git]
    enable = false
    ```

**git.auto_commit**
:   When `true`, Tildr automatically runs `git add -A` and `git commit` after **add**,
    **restore**, **del**, **mv**, and **secret**. Does not affect **apply**, **unlink**, **git**, or **sync**.
    Default: `true`

    Example:
    ```toml
    [git]
    auto_commit = true
    ```

## [crypto]

The `[crypto]` section controls GPG encryption for sensitive files.

**crypto.mode**
:   Encryption mode used by **tildr secret**. Accepted values:

    `symmetric` — passphrase-only encryption using GPG symmetric AES-256. No key pair required.
    The same passphrase must be used to decrypt on any machine.

    `asymmetric` — encryption using an existing GPG key pair. The recipient is set via
    `crypto.gpg_key`. Decryption uses the private key and requires no additional passphrase
    (subject to GPG Agent caching).

    Default: `symmetric`

    Example:
    ```toml
    [crypto]
    mode = "symmetric"
    ```

**crypto.gpg_key**
:   GPG key ID or email address used when `crypto.mode = "asymmetric"`. When empty, Tildr
    prompts interactively on first use and saves the chosen key to this field automatically.
    Ignored when `crypto.mode = "symmetric"`.
    Default: empty

    Example:
    ```toml
    [crypto]
    mode = "asymmetric"
    gpg_key = "william@email.com"
    ```

# FULL EXAMPLE

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

# PATH RESOLUTION RULES

Most path arguments to Tildr commands are interpreted relative to `$HOME`.

- `tildr add .config/nvim/init.lua` resolves to `$HOME/.config/nvim/init.lua`
- `tildr add ~/notes/todo.md` resolves via the home shorthand
- Absolute paths are accepted only if they still point inside `$HOME`

For `tildr init --repo`, the path may be provided as:

- `~/...` for a home-relative path
- An absolute path inside `$HOME`
- A relative path resolved from the current working directory, which must end up inside `$HOME`

# IGNORE SUPPORT

Tildr supports a repository-level `.tildrignore` file for excluding paths from repository scans.

- The file must live at the root of the Tildr repository
- Ignore rules are applied when scanning repository contents
- Patterns follow gitignore-style matching semantics

Example:

```
*.log
cache/
.DS_Store
```

# INTERNALLY EXCLUDED PATHS

During repository scans, Tildr always excludes the following entries regardless of `.tildrignore`:

- `.git`
- `.gitignore`
- `.tildrignore`
- `.tildr/` — internal directory containing all Tildr configuration files
- `.DS_Store`
- `Thumbs.db`
- `.gitkeep`
- Files ending in `.bak`
- Files ending in `.tmp`
- Files ending in `.swp`
- Files ending in `~`

# ENVIRONMENT VARIABLES

**NO_COLOR**
:   When set, disables all colored output regardless of `core.color` config.

**EDITOR**
:   Editor used by `tildr edit`. Falls back to `$VISUAL`, then `nano`.

**VISUAL**
:   Alternative editor used by `tildr edit` if `$EDITOR` is not set.

**PAGER**
:   Pager used by `tildr cat --less` and `tildr status --less`. Falls back to `less -RFX`.

# FILES

**~/.config/tildr/config.toml**
:   User configuration file.

**~/.dotfiles/.tildr/**
:   Internal Tildr directory containing encrypted-items, encrypted.gpg, and groups.json.

**~/.dotfiles/.tildrignore**
:   User-defined ignore patterns.

# SEE ALSO

**tildr(1)**, **tildr-commands(1)**, **tildr-security(1)**, **tildr-plugins(1)**

# AUTHORS

Maintained by the Tildr contributors.
Source code and issue tracker: <https://github.com/orbitbits/tildr>
