# Appwrite Rust Playground

A hands-on CLI playground for exploring the [Appwrite](https://appwrite.io) API using the official [`appwrite`](https://crates.io/crates/appwrite) Rust SDK. Run interactive demos against a real Appwrite instance, with colored output, spinners, and automatic resource cleanup.

## Covered APIs

| Service | What it demos |
|---------|---------------|
| **Health** | Server status, database health, storage health, server time |
| **TablesDB** | Database + table CRUD, string & integer columns, indexes, row CRUD with query filters |
| **Storage** | Bucket CRUD, file upload / download / rename |
| **Users** | User CRUD, name updates, preference management |
| **Functions** | Function CRUD, Node.js deployment, env variables, sync & async execution |

> Every mutating demo creates temporary resources and cleans them up automatically when finished.

## Requirements

- **Rust** toolchain (edition 2024)
- An **Appwrite** instance &mdash; [Appwrite Cloud](https://cloud.appwrite.io) works out of the box
- [**Appwrite CLI**](https://appwrite.io/docs/tooling/command-line/installation) installed and logged in (`appwrite login`) &mdash; only needed for automated setup
- **jq** (`brew install jq` on macOS) &mdash; only needed for automated setup
- **tar** &mdash; only needed for the `functions` demo

## Setup

### Automated (recommended)

```bash
./setup.sh                       # creates project + API key via Appwrite CLI
set -a && source .env && set +a  # load credentials into your shell
```

### Manual

```bash
cp .env.example .env   # fill in your credentials
set -a && source .env && set +a
```

| Variable | Required | Description |
|----------|----------|-------------|
| `APPWRITE_ENDPOINT` | Yes | e.g. `https://cloud.appwrite.io/v1` |
| `APPWRITE_PROJECT_ID` | Yes | Your Appwrite project ID |
| `APPWRITE_API_KEY` | Yes | Server API key with required scopes |
| `APPWRITE_SELF_SIGNED` | No | Set to `true` for self-signed TLS certs |
| `APPWRITE_SAMPLE_FILE` | No | Path to upload file (default: `resources/sample-upload.txt`) |
| `APPWRITE_FUNCTION_SOURCE_DIR` | No | Path to function source (default: `resources/functions/hello-node`) |

## Run

```bash
cargo run                          # interactive demo picker
cargo run -- tablesdb              # run a single demo
cargo run -- tablesdb storage      # run multiple demos
cargo run -- all                   # run everything
cargo run -- help                  # show available commands
```

When run without arguments, an interactive multi-select menu lets you pick which demos to run.

## Notes

- Function deployments are built from [`resources/functions/hello-node`](./resources/functions/hello-node) and packaged into a temporary `.tar.gz` archive at runtime.
- If your Appwrite instance doesn't support a given service or API scope, that specific demo will fail while the others remain available.
- A few workarounds for SDK bugs are annotated with `TODO(sdk-fix)` in the source &mdash; see [sdk-for-rust#10](https://github.com/appwrite/sdk-for-rust/issues/10) for details.
