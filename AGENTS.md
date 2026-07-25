# Tildr Codex Instructions

## Project

Tildr is a declarative HOME directory manager written in Rust. It manages
dotfiles through symlinks stored in a Tildr repository.

## Rust

- Use stable Rust with edition 2024.
- Prefer `anyhow::Result` for command-level errors.
- Avoid `unwrap()` in production code.
- Keep modules small and reuse existing domain types/helpers.
- Prefer iterators when they make the code clearer.
- Avoid unnecessary allocation.
- Keep user-facing behavior covered by focused tests.

## Repository Model

- Managed dotfiles live under the Tildr repository, not directly under HOME.
- `common/` represents files with no profile; show it to users as `no profile`
  when a profile label is needed.
- `profiles/<name>/` stores profile-specific variants.
- `common/` and `profiles/<name>/` are storage roots. User-facing commands should
  accept logical HOME paths such as `~/.bashrc`, `$HOME/.bashrc`, `.bashrc`, or
  paths relative to the current directory under HOME.
- Do not require users to type storage paths such as `common/.bashrc` or
  `profiles/linux/.bashrc` unless the command explicitly asks for source paths.

## Documentation

When changing Rust behavior, check whether documentation must be updated.
Documentation means both:

- `docs/site/*.md`
- `docs/man/*.md`

Generated man pages live in `docs/man/dist/*.1`. Regenerate them with `make man`
when the source man Markdown changes.

## Validation

Before finishing code changes, prefer:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
```

Use `make check` when the Makefile workflow is the better fit.

For documentation/version/release changes, also consider:

```sh
make docs-version-check
make man
git diff --check
```

## Commits

- Never commit without an explicit user request.
- Use conventional commits in English.
- When docs and code are both changed and the user asks for separated commits,
  commit documentation first and Rust code second.
- Do not include unrelated or untracked user files in commits.

## Worktree Safety

- Do not revert changes you did not make.
- If unrelated files are dirty, leave them alone.
- Prefer non-destructive Git commands.
- Do not use `git reset --hard` or `git checkout --` unless explicitly requested.
