# Git Hooks

To use, run the following command only once in the project directory:

```sh
git config core.hooksPath hooks
```

## Available Hooks

### `pre-commit`

Runs `make check` before each commit. This performs a non-mutating format check, Clippy with warnings denied, and the workspace test suite.

### `commit-msg`

Validates commit messages against the [Conventional Commits](https://www.conventionalcommits.org/) specification. Allowed types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `style`, `perf`, `ci`, `build`.

## Development

See [DEVELOPMENT.md](../DEVELOPMENT.md) for full development workflow documentation.
