<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<p align="center">
  <img src=".github/brand/logo-text/compact/tildr-variation-3.svg" alt="tildr" width="180"/>
</p>

<h2 align="center">Manage your HOME files and directories with symlinks and Git.</h2>

[![Version](https://img.shields.io/github/v/release/orbitbits/tildr.svg)](https://github.com/orbitbits/tildr/releases)
[![License](https://img.shields.io/badge/license-Elastic%20License%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/MSRV-1.90.0-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey.svg)]()
[![Build](https://img.shields.io/github/actions/workflow/status/orbitbits/tildr/release.yml?branch=main)](https://github.com/orbitbits/tildr/actions)
[![Downloads](https://img.shields.io/github/downloads/orbitbits/tildr/total.svg)](https://github.com/orbitbits/tildr/releases)

---

## Quick Start


```sh
# Install
curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh

# Initialize
tildr init

# Add your first dotfile
tildr add .bashrc
tildr apply
tildr sync
```

---

## Introduction

**Manage, reproduce, and control everything in your `$HOME` — declaratively.**

`Tildr` is a fast and minimalist CLI that lets you define the desired state of your HOME directory and automatically
enforce it.

Instead of manually copying dotfiles, syncing folders, or rebuilding your environment from memory, you describe how your
`$HOME` should look — and `Tildr` makes your system converge to that state.

> More powerful than *stow*. Simpler than *chezmoi*.

---

## Why Tildr?

Traditional dotfile tools manage files. **Tildr** manages your HOME state.

It treats your HOME directory as a reproducible environment—not just a collection of configuration files.

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

## About this repository

This public repository exists to:

* Provide verified and reproducible source code and binary versions
* Serve as the official download location
* Publish the `Tildr` logo for use on the official OrbitBits website
* Receive feedback, bug reports, and suggestions from users

All binaries published here are automatically compiled through a controlled CI pipeline to ensure authenticity and
integrity.

For complete documentation and usage guides, please visit the official pages below.

## Official page

[https://orbitbits.github.io/tildr/](https://orbitbits.github.io/tildr/)

## Documentation

[https://orbitbits.github.io/tildr/documentation/](https://orbitbits.github.io/tildr/documentation/)

## Verifying Releases

All binaries are signed and can be verified.
See [SECURITY.md](SECURITY.md) for full verification instructions.

## Community

* [Contributing](CONTRIBUTING.md)
* [Development](DEVELOPMENT.md)
* [License Third-Party](LICENSE-THIRD-PARTY.md)

## LICENSE

"Tildr is source-available under the Elastic License 2.0. You may use, modify
and contribute freely, but you may not sell or redistribute Tildr as a product
or service."

See more at: [License](LICENSE)

---

© [OrbitBits](https://orbitbits.github.io) - All rights reserved.
