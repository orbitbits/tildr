# Environment Variables

Tildr reads a small set of environment variables for terminal behavior, editor integration, path resolution, and development workflows.

## Runtime Variables

### `NO_COLOR`

Disables colored output when set to any value. Tildr follows the [NO_COLOR standard](https://no-color.org/).

```sh
NO_COLOR=1 tildr status
```

When `core.color = false` is set in `config.toml`, Tildr sets `NO_COLOR=1` at startup.

### `EDITOR`

Primary editor used by `tildr edit`.

```sh
EDITOR=vim tildr edit ~/.bashrc
```

### `VISUAL`

Fallback editor used by `tildr edit` when `EDITOR` is not set.

If neither `EDITOR` nor `VISUAL` is set, Tildr falls back to `nano`.

### `PAGER`

Pager program for long output. Tildr defaults to `less -RFX`.

```sh
PAGER='less -R' tildr list --less
```

## Path Resolution

### `HOME`

Tildr resolves `$HOME` in user-provided paths for commands such as `add`, `restore`, `unlink`, `edit`, `cat`, and `source-path`.

These forms are equivalent when they point to the same file:

```sh
tildr add ~/.config/starship.toml
tildr add "$HOME/.config/starship.toml"
tildr add .config/starship.toml
```

Internally, commands use the configured Tildr context home path instead of reading `HOME` directly whenever possible. This keeps tests and alternate contexts deterministic.

## Locale Detection

### `LC_ALL`, `LC_CTYPE`, `LANG`

Checked in that order to determine whether the terminal supports UTF-8 encoding.

When UTF-8 is detected, Tildr displays Unicode symbols. Otherwise, it falls back to ASCII symbols.

## Display Detection

### `DISPLAY`, `WAYLAND_DISPLAY`

On Linux, used to detect whether a graphical display server is available.

This affects features that can open graphical pickers, such as group file selection when `tildr group add <name>` is run without `--files`.

## Release and CI Variables

The release workflow uses GitHub Actions secrets to sign release artifacts:

- `GPG_PRIVATE_KEY`
- `GPG_PASSPHRASE`
- `GPG_KEY_ID`

These are not required for local development.

GitHub Actions also provides `GITHUB_TOKEN` automatically for release creation and changelog operations.
