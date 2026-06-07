# Contributing to TelaRex

Thank you for your interest in TelaRex! This document covers how to report bugs, request features, and submit changes.

## Reporting Bugs

Open a [GitHub Issue](https://github.com/cristiano-s-r-filho/telarex/issues/new?template=bug_report.md) with:

- **Steps to reproduce** — minimal, exact sequence
- **Expected vs actual behaviour**
- **Environment** — OS, terminal emulator, `trex --version`
- **Logs** — run with `RUST_LOG=debug trex` and paste relevant output

## Feature Requests

Open a [Feature Request](https://github.com/cristiano-s-r-filho/telarex/issues/new?template=feature_request.md) describing the use case and why existing workflows don't cover it.

## Pull Requests

1. **Fork** the repo and create a branch from `main`.
2. **Run tests** before opening: `cargo test --workspace`
3. **Run the linter**: `cargo clippy --workspace -- -D warnings`
4. **Ensure docs build**: `cargo doc --no-deps --workspace`
5. Open a PR with a clear title and description linking to any related issues.

### Code style

- Follow the existing patterns in the file you're editing.
- Use `cargo fmt` before committing.
- Keep doc comments concise — explain *what*, not *how*.
- Avoid adding new dependencies without discussion.

## Getting Help

- Open a [Discussion](https://github.com/cristiano-s-r-filho/telarex/discussions) for questions.
- Read the [Beta Guide](docs/BETA_GUIDE.md) for setup and workflow help.
