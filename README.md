# Appwrite Rust Playground

Appwrite playground is a simple way to explore the Appwrite API and the
[`appwrite`](https://crates.io/crates/appwrite) Rust SDK. This repo is modeled
after Appwrite's other playgrounds, with a Rust-first CLI that runs isolated
examples for multiple services.

## Covered APIs

- Health
  - Get overall status
  - Get database status
  - Get storage status
  - Get server time
- TablesDB
  - Create database
  - List databases
  - Get database
  - Update database
  - Create table
  - List tables
  - Get table
  - Update table
  - Create columns
  - Create index
  - Create row
  - List rows
  - Get row
  - Update row
- Storage
  - Create bucket
  - List buckets
  - Get bucket
  - Update bucket
  - Upload file
  - List files
  - Get file
  - Update file
  - Download file
- Users
  - Create user
  - List users
  - Get user
  - Update name
  - Get prefs
  - Update prefs
- Functions
  - Create function
  - List functions
  - Get function
  - Update function
  - Upload deployment
  - List deployments
  - Create variable
  - List variables
  - Get variable
  - Update variable
  - Execute function sync
  - Execute function async
  - List executions

## Requirements

- Rust toolchain installed
- An Appwrite instance (or [Appwrite Cloud](https://cloud.appwrite.io))
- [Appwrite CLI](https://appwrite.io/docs/tooling/command-line/installation) installed and logged in (`appwrite login`)
- `jq` installed (`brew install jq` on macOS)
- `tar` available on your machine if you want to run the `functions` demo

## Setup

### Automated (recommended)

The setup script uses the Appwrite CLI to create a project, generate an API key
with all the required scopes, and write everything to `.env`:

```bash
./setup.sh
```

Then load the environment:

```bash
set -a && source .env && set +a
```

### Manual

Copy the example env file and fill in your Appwrite credentials:

```bash
cp .env.example .env
```

Then export it into your shell:

```bash
set -a
source .env
set +a
```

You can also export the variables manually:

```bash
export APPWRITE_API_KEY="your-api-key"
export APPWRITE_ENDPOINT="https://cloud.appwrite.io/v1"
export APPWRITE_PROJECT_ID="your-project-id"
export APPWRITE_SELF_SIGNED=false
```

## Run

Default command:

```bash
cargo run
```

When run without arguments, an interactive menu lets you pick which demos to run.

Run a single mutating demo:

```bash
cargo run -- tablesdb
cargo run -- storage
cargo run -- users
cargo run -- functions
```

Run multiple demos:

```bash
cargo run -- tablesdb storage users
```

Run everything:

```bash
cargo run -- all
```

Show available commands:

```bash
cargo run -- help
```

## Notes

- All demos except `health` create temporary Appwrite resources and then try to
  delete them again at the end.
- Storage uploads use [`resources/sample-upload.txt`](./resources/sample-upload.txt)
  by default.
- Function deployments are built from
  [`resources/functions/hello-node`](./resources/functions/hello-node) and
  packaged into a temporary `.tar.gz` archive at runtime.
- If your Appwrite instance doesn't support a given product or API scope yet,
  that specific demo will fail while the others remain available.
