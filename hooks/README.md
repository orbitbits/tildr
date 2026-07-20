# Git Hooks

To use, run the following command only once in the project directory:

```sh
git config core.hooksPath hooks
```

## Available Hooks

### `pre-commit`

Runs `make build` (`cargo fmt --all` + `cargo build`) before each commit. Aborts the commit if the build fails or if formatting changes files under `crates/`.

### `commit-msg`

Validates commit messages against the [Conventional Commits](https://www.conventionalcommits.org/) specification. Allowed types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `style`, `perf`, `ci`, `build`.

## Development

See [DEVELOPMENT.md](../DEVELOPMENT.md) for full development workflow documentation.
