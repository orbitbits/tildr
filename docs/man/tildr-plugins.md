---
title: TILDR-PLUGINS
section: 1
header: User Commands
footer: Tildr
date: 2026
---

# NAME

tildr-plugins — file manager plugins for Tildr

# SYNOPSIS

Tildr provides native integration with popular Linux file managers through plugins,
allowing you to manage files directly from the context menu without using the terminal.

# DESCRIPTION

Both plugins expose the same three operations via a **Tildr** submenu in the context menu:

**Add**
:   Add the selected file(s) to the Tildr repository and replace them with symlinks.

**Unlink**
:   Remove the symlink(s) from `$HOME` without touching repository content.

**Restore**
:   Move the file(s) back from the repository into `$HOME` and remove the symlinks.

Multiple file selection is supported in both plugins. All selected files are passed to the Tildr command in a single invocation.

# NAUTILUS

Integration with Nautilus (GNOME Files) is done via **nautilus-python**,
a Python extension API that allows adding custom items to the context menu.

## Requirements

One of the following packages must be installed:

- `python-nautilus` — Arch Linux
- `python3-nautilus` — Debian, Ubuntu, Linux Mint
- `nautilus-python` — Fedora

## Features

- Context menu submenu **Tildr → Add / Unlink / Restore**
- Supports single and multiple file selection
- Available only for files and symlinks, not directories
- Only appears when inside the user's home directory (`~/`)

## Manual Installation

```sh
mkdir -p ~/.local/share/nautilus/python-extensions
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/nautilus/tildr.py \
  -o ~/.local/share/nautilus/python-extensions/tildr.py
nautilus -q
```

## Plugin Location

`~/.local/share/nautilus/python-extensions/tildr.py`

# DOLPHIN

Integration with Dolphin (KDE) is done via **KIO Service Menus**,
a `.desktop` file mechanism that adds custom actions to the context menu.
No additional dependencies are required beyond Dolphin itself.

## Features

- Context menu submenu **Tildr → Add / Unlink / Restore**
- Supports single and multiple file selection
- Available for files and symlinks

## Manual Installation

```sh
mkdir -p ~/.local/share/kio/servicemenus
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/dolphin/tildr.desktop \
  -o ~/.local/share/kio/servicemenus/tildr.desktop
```

No restart is required. Dolphin picks up new service menus automatically.

## Plugin Location

`~/.local/share/kio/servicemenus/tildr.desktop`

# AUTOMATIC INSTALLATION

Both plugins are installed automatically by the Tildr installer script
when the respective file manager is detected on the system:

```sh
curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh
```

If the file manager is not installed at the time of Tildr installation,
the plugin can be installed manually at any time using the commands above.

# UNINSTALLATION

## Nautilus

```sh
rm ~/.local/share/nautilus/python-extensions/tildr.py
nautilus -q
```

## Dolphin

```sh
rm ~/.local/share/kio/servicemenus/tildr.desktop
```

No restart is required. Dolphin picks up changes automatically.

# TROUBLESHOOTING

## Submenu does not appear

- Ensure the plugin file exists in the correct location
- Check that the file has read permissions
- For Nautilus: restart with `nautilus -q` and reopen
- For Dolphin: restart Dolphin or run `kbuildsycoca5`

## "Command not found" when clicking a menu action

- Ensure `tildr` is installed and available in `PATH`
- Open a terminal and run `which tildr` to verify

## Plugin appears outside of $HOME

This should not happen. The Nautilus plugin checks the current directory and only shows the submenu when inside `$HOME`. If this occurs, please open an issue.

# SEE ALSO

**tildr(1)**, **tildr-config(1)**, **tildr-commands(1)**, **tildr-security(1)**
