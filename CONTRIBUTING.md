# Contributing to Tildr

Thank you for your interest in contributing to **Tildr**! This document describes how to participate in the project in an organized and transparent way.

---

## Before contributing

Tildr is a project of the **OrbitBits** organization, licensed under the [GNU Affero General Public License v3.0](LICENSE). By submitting any contribution, you agree to the terms described in this document.

---

## Contributor License Agreement (CLA)

To maintain the legal integrity of the project and ensure that OrbitBits can continue developing and distributing Tildr, all contributors must accept the following agreement:

> **By submitting a contribution to this repository (via Pull Request, patch, code issue, or any other means), you declare and agree that:**
>
> 1. **License Assignment:** You grant OrbitBits a perpetual, irrevocable, worldwide, non-exclusive, royalty-free, and sublicensable license to use, reproduce, modify, distribute, sublicense, and incorporate your contribution into Tildr and derivative works, in any format.
>
>
> 2. **Copyright Retained:** You remain the author of your contribution. OrbitBits does not claim ownership of your original work, only the usage rights described above.
>
>
> 3. **Originality:** You declare that your contribution is your original work, or that you have the necessary rights to submit it under these terms.
>
>
> 4. **Absence of encumbrances:** Your contribution is not subject to any agreement, patent, or third-party right that conflicts with the terms above.
>
>
> 5. **License acknowledgment:** You understand that Tildr is distributed under the GNU Affero General Public License v3.0 and that your contribution will become part of the project under that same license.

Acceptance occurs **implicitly** by opening a Pull Request in this repository.

---

## How to Contribute

### 1. Report Bugs

Open an issue describing:

- What you expected to happen
- What actually happened
- Tildr version, operating system, and steps to reproduce it
- Whether profiles, groups, secrets, or generated man pages are involved

### 2. Suggest Improvements

Open an issue labeled `enhancement` describing your idea before implementing it. This avoids double work and ensures alignment with the project direction.

### 3. Submitting Code (Pull Request)

1. Fork the repository
2. Create a descriptive branch: `git checkout -b feat/feature-name`
3. Make clear and atomic commits
4. Ensure the code compiles, lints, and tests pass
5. Update documentation when behavior, commands, flags, output, profiles, groups, or security workflows change
6. Open a Pull Request describing what was done and why

See [DEVELOPMENT.md](DEVELOPMENT.md) for development setup, build commands, and project architecture.

### 4. Code Standards

- Follow the style already adopted in the project
- Write commit messages in English using Conventional Commits, such as `feat: add source path lookup` or `fix(group): normalize paths`
- Document public functions when necessary
- Prefer small, focused changes that can be reviewed independently
- Add or update tests for bug fixes and user-visible behavior changes
- Keep generated artifacts in sync when the repository already tracks them

### 5. Local Checks

Before opening a Pull Request, run:

```sh
make check
```

When documentation under `docs/man/*.md` changes, also run:

```sh
make man
git diff -- docs/man docs/man/dist
```

The generated files in `docs/man/dist` are committed because installers consume them.

---

## Code of Conduct

We expect all contributors to maintain a respectful and collaborative environment. Disrespectful, discriminatory, or abusive behavior will not be tolerated and may result in the repository being locked.

---

## Questions?

Open an issue with the label `question` or contact the organization on GitHub: [github.com/orbitbits](https://github.com/orbitbits).

---

*Copyright (c) 2026 OrbitBits. All rights reserved.*
