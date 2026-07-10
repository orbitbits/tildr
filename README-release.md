<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<p align="center">
  <img src=".github/brand/logo-text/compact/tildr-variation-3.svg" alt="tildr" width="180"/>
</p>

<h2 align="center">Manage your HOME files and directories with symlinks and Git.</h2>

[![Version](https://img.shields.io/badge/version-0.0.0-blue.svg)](https://github.com/orbitbits/tildr/releases)
![Rust](https://img.shields.io/badge/Rust-built-orange)

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

Traditional dotfile tools focus on files. `Tildr` focuses on **state**.

It treats your HOME directory as a reproducible environment — not just a collection of configs.

With `Tildr`, you can:

* Define the structure and contents of your `$HOME`
* Keep files and directories consistently in sync
* Recreate your environment reliably at any time
* Eliminate configuration drift
* Manage more than dotfiles — manage your **entire home state**

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
