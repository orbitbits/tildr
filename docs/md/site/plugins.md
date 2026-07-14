---
layout: doc
part: 6
section: Integrations
menu: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: File Manager Plugins
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/plugins/
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
