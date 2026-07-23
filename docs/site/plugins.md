---
layout: doc
part: 6
section: Integrations
menu: tildr
version: "0.3.1"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: File Manager Plugins
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.3.1/plugins/
---

## Plugins

Tildr provides native integration with popular Linux file managers through plugins,
allowing you to manage your files directly from the context menu without using the terminal.

---

### Nautilus

Integration with Nautilus (GNOME Files) is done via **nautilus-python**,
a Python extension API that allows adding custom items to the context menu.

#### Requirements

| Distribution | Package |
|--------------|---------|
| Arch Linux | `python-nautilus` |
| Debian / Ubuntu / Mint | `python3-nautilus` |
| Fedora | `nautilus-python` |

Install with your package manager:

```sh
# Arch Linux
sudo pacman -S python-nautilus

# Debian/Ubuntu
sudo apt install python3-nautilus

# Fedora
sudo dnf install nautilus-python
```

#### Features

* Context menu with submenu **Tildr → Add / Unlink / Restore**
* Supports single and multiple file selection
* Available only for files and symlinks (not directories)
* Only appears inside the user's home directory (`~/`)

#### Manual Installation

```sh
mkdir -p ~/.local/share/nautilus/python-extensions
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/nautilus/tildr.py \
  -o ~/.local/share/nautilus/python-extensions/tildr.py
nautilus -q
```

#### Restart Nautilus

After installation, restart Nautilus:

```sh
nautilus -q
```

Or log out and log back in.

#### Troubleshooting

**Plugin not appearing in context menu:**

1. Verify the plugin is installed:
   ```sh
   ls -la ~/.local/share/nautilus/python-extensions/tildr.py
   ```

2. Check Nautilus version:
   ```sh
   nautilus --version
   ```

3. Restart Nautilus:
   ```sh
   nautilus -q
   ```

4. Check for errors in the terminal:
   ```sh
   nautilus --no-desktop 2>&1 | grep -i tildr
   ```

**Permission denied errors:**

Ensure the plugin file is executable:

```sh
chmod +x ~/.local/share/nautilus/python-extensions/tildr.py
```

---

### Dolphin

Integration with Dolphin (KDE) is done via **KIO Service Menus**,
a `.desktop` file mechanism that adds custom actions to the context menu.
No additional dependencies are required beyond Dolphin itself.

#### Features

* Context menu with submenu **Tildr → Add / Unlink / Restore**
* Supports single and multiple file selection
* Available for files and symlinks

#### Manual Installation

```sh
mkdir -p ~/.local/share/kio/servicemenus
curl -fsSL https://raw.githubusercontent.com/orbitbits/tildr/main/tools/plugins/dolphin/tildr.desktop \
  -o ~/.local/share/kio/servicemenus/tildr.desktop
```

> No restart required — Dolphin picks up new service menus automatically.

#### Troubleshooting

**Plugin not appearing in context menu:**

1. Verify the desktop file is installed:
   ```sh
   ls -la ~/.local/share/kio/servicemenus/tildr.desktop
   ```

2. Check the desktop file syntax:
   ```sh
   cat ~/.local/share/kio/servicemenus/tildr.desktop
   ```

3. Restart Dolphin:
   ```sh
   dolphin --quit
   dolphin
   ```

**Desktop file validation:**

```sh
desktop-file-validate ~/.local/share/kio/servicemenus/tildr.desktop
```

---

### Automatic Installation

Both plugins are installed automatically by the Tildr installer script
when the respective file manager is detected on the system:

```sh
curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh
```

If the file manager is not installed at the time of Tildr installation,
you can install the plugin manually at any time using the commands above.

---

### Plugin Limitations

| Feature | Nautilus | Dolphin |
|---------|----------|---------|
| **Add** | Yes | Yes |
| **Unlink** | Yes | Yes |
| **Restore** | Yes | Yes |
| **Directories** | No | No |
| **Outside $HOME** | No | No |
| **Multiple selection** | Yes | Yes |

---

### Disabling Plugins

To remove a plugin:

**Nautilus:**

```sh
rm ~/.local/share/nautilus/python-extensions/tildr.py
nautilus -q
```

**Dolphin:**

```sh
rm ~/.local/share/kio/servicemenus/tildr.desktop
```

No restart required for Dolphin.

---

### Alternative File Managers

If your file manager is not supported by a native plugin, you can:

1. Use the CLI directly (`tildr add`, `tildr unlink`, `tildr restore`)
2. Create a custom integration using the Tildr CLI
3. Request support by opening an issue on the [GitHub repository](https://github.com/orbitbits/tildr/issues)
