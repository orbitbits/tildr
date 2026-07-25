---
name: tildr-rust-maintainer
description: Use when modifying Tildr Rust code, command behavior, profile/common path handling, symlink logic, config parsing, git sync, tests, or refactors in this repository.
---

# Tildr Rust Maintainer

## Workflow

1. Read `AGENTS.md` first.
2. Locate the command or domain boundary with `rg` and existing tests.
3. Preserve the repository model:
   - `common/` means `no profile`.
   - `profiles/<name>/` contains profile variants.
   - User inputs should resolve from HOME/logical paths, including `~`, `$HOME`,
     absolute paths under HOME, and cwd-relative paths under HOME.
4. Reuse existing helpers for target/path resolution before adding new parsing.
5. Add or update focused tests for the changed behavior.
6. Check whether `docs/site/*.md` and `docs/man/*.md` need updates.

## Rust Style

- Prefer typed helpers over ad hoc string manipulation.
- Avoid `unwrap()` in production code.
- Keep command functions small enough to review.
- Make error messages actionable for CLI users.
- Keep behavior deterministic across Linux and macOS.

## Validation

Run the narrowest useful tests while iterating, then the broader checks before
finishing:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
```
