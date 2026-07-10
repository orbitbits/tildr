# Git Hooks

To use, run the following command only once in the project directory:

```sh
git config core.hooksPath hooks
```

## Available Hooks

### `pre-commit`

Runs `make build` (cargo fmt + cargo build) before each commit. Aborts the commit if the build fails or if the build generates uncommitted changes (e.g., formatted code or generated man pages).

### `commit-msg`

Validates commit messages against the [Conventional Commits](https://www.conventionalcommits.org/) specification. Allowed types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `style`, `perf`, `ci`, `build`, `revert`.

## Development

See [DEVELOPMENT.md](../DEVELOPMENT.md) for full development workflow documentation.
