---
name: tildr-docs-release
description: Use when updating Tildr documentation, man pages, docs version metadata, release workflows, changelog generation, or conventional release commits.
---

# Tildr Docs And Release

## Documentation Rules

- "Documentation" means both `docs/site/*.md` and `docs/man/*.md`.
- Update generated man pages in `docs/man/dist/*.1` with `make man` when
  `docs/man/*.md` changes.
- Keep `docs/site/*.md` front matter versions synchronized with the workspace
  version using `make docs-version`.
- Validate docs version drift with `make docs-version-check`.

## Release Expectations

- `Cargo.toml` workspace version is the source of truth.
- Release builds should preserve generated man pages and release artifacts.
- Release workflows should produce checksums and signatures when configured.
- Avoid workflow steps that push stale branches without integrating remote state.

## Commit Pattern

When requested to commit documentation and code separately:

1. Commit docs first with `docs: ...`.
2. Commit Rust/workflow changes after with `feat:`, `fix:`, `refactor:`, or
   `chore:` as appropriate.

Use an explanatory commit body for broad documentation or release automation
changes.

## Validation

```sh
make docs-version-check
make man
git diff --exit-code -- docs/site docs/man/dist
git diff --check
```
