---
name: tildr-quality-review
description: Use when doing a broad Tildr code review, finding Rust inconsistencies, reducing duplication, improving idiomatic Rust, or checking GitHub Actions and project hygiene.
---

# Tildr Quality Review

## Review Focus

- Look first for bugs, regressions, path/profile inconsistencies, missing tests,
  and cross-platform issues.
- Treat `common/` as storage for `no profile`; do not expose storage paths in
  normal user-facing flows unless the command explicitly needs source paths.
- Check that commands accepting file targets support logical HOME resolution:
  `~`, `$HOME`, absolute HOME paths, cwd-relative paths under HOME, and plain
  relative paths.
- Keep docs and tests aligned with behavior.

## Refactor Guidance

- Prefer small helpers, enums, and structs where they remove real duplication.
- Keep public CLI behavior stable unless the task asks to change it.
- Avoid broad rewrites that make review harder.
- Keep platform-sensitive filesystem and symlink logic covered by tests.

## Validation

Start with targeted tests for the touched subsystem, then run:

```sh
make check
git diff --check
```
