# Contributing

Thanks for your interest in contributing to the Appwrite Rust Playground!

## Getting Started

1. Fork and clone the repo
2. Run `./setup.sh` or manually create a `.env` file (see [README](./README.md#setup))
3. Load your environment: `set -a && source .env && set +a`
4. Verify everything works: `cargo run -- all`

## Development

### Building

```bash
cargo build
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Running a specific demo

```bash
cargo run -- health
cargo run -- tablesdb
```

## Adding a New Demo

1. Add a variant to the `Demo` enum in `src/main.rs`
2. Implement `label()`, `emoji()`, and `description()` for your variant
3. Write a `run_<service>_demo()` method on `Playground`
4. Add a corresponding cleanup function that tears down any resources created
5. Update the `README.md` "Covered APIs" table
6. If the setup script needs new API scopes, add them to `setup.sh`

## Pull Requests

- Branch off `main` and open a PR back to `main`
- Keep PRs focused — one feature or fix per PR
- Make sure `cargo build` and `cargo clippy` pass
- Test your changes against a real Appwrite instance

## SDK Workarounds

Some Appwrite Rust SDK bugs are worked around in the codebase (search for `TODO(sdk-fix)`). If you encounter a new SDK bug, add a similar annotated workaround and note it in your PR description. Upstream issues go to [appwrite/sdk-for-rust](https://github.com/appwrite/sdk-for-rust/issues).

## Code Style

- Follow existing patterns in `src/main.rs`
- Use `anyhow::Context` for error messages
- Use the `print_json()` helper for API responses
- Clean up all resources your demo creates, even on error paths
