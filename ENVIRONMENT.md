# Environment Variables for tildr

`Tildr` supports the following environment variables to control its behavior and user interface.

## User Interface

### `NO_COLOR`

Disables all colored output when set to any value. Implements the [NO_COLOR standard](https://no-color.org/).

```sh
NO_COLOR=1 tildr status
```

When `core.color = false` is set in `config.toml`, Tildr automatically sets `NO_COLOR=1` at startup.

### `EDITOR`

Specifies the text editor used by `tildr edit`. Falls back to `VISUAL`, then to `nano`.

```sh
EDITOR=vim tildr edit .bashrc
```

### `VISUAL`

Fallback editor for `tildr edit` when `EDITOR` is not set. Falls back to `nano`.

### `PAGER`

Pager program for long output (e.g., `tildr info license`). Defaults to `less -RFX`.

```sh
PAGER=bat tildr cat --less
```

## Locale Detection

### `LC_ALL`, `LC_CTYPE`, `LANG`

Checked (in that order) to determine whether the terminal supports UTF-8 encoding. When UTF-8 is detected, Tildr displays Unicode icons (checkmarks, arrows, etc.). Otherwise, it falls back to ASCII symbols.

## Display Detection

### `DISPLAY`, `WAYLAND_DISPLAY`

On Linux, checked to detect whether a graphical display server (X11 or Wayland) is available. Used internally for feature detection.
