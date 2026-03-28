<p align="center">
  <a href="https://appwrite.io"><img src="https://avatars.githubusercontent.com/u/25003669" alt="Appwrite" height="80" style="border-radius:16px"></a>
  &nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://www.rust-lang.org"><img src="https://www.rust-lang.org/logos/rust-logo-512x512.png" alt="Rust" height="80"></a>
</p>

<h1 align="center">Appwrite Rust Playground</h1>

<p align="center">
  A hands-on CLI playground for exploring the <a href="https://appwrite.io">Appwrite</a> API using the official <a href="https://crates.io/crates/appwrite"><code>appwrite</code></a> Rust SDK.<br>
  Run interactive demos against a real Appwrite instance, with colored output, spinners, and automatic resource cleanup.
</p>

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
./setup.sh          # auto-detects org, creates project + API key, writes .env
cargo run -- all    # .env is loaded automatically
```

The script accepts optional flags to override defaults:

```bash
./setup.sh --org-id <id> --project-name my-app --project-id my-app-id
./setup.sh -h       # show all options
```

### Manual

```bash
cp .env.example .env   # fill in your credentials
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
- Uses the Appwrite Rust SDK v0.2.0 with native methods for all API calls.
