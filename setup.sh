#!/usr/bin/env bash
#
# setup.sh — Bootstrap the Appwrite Rust Playground
#
# Uses `appwrite init project` to create/link an Appwrite project,
# then creates an API key with all required scopes and writes
# credentials to .env so you can immediately run `cargo run -- all`.
#
# Prerequisites:
#   1. Appwrite CLI installed  (npm install -g appwrite-cli)
#   2. Logged in               (appwrite login)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ---------- helpers ----------------------------------------------------------

info()  { printf '\033[1;34m▸ %s\033[0m\n' "$*"; }
ok()    { printf '\033[1;32m✔ %s\033[0m\n' "$*"; }
err()   { printf '\033[1;31m✘ %s\033[0m\n' "$*" >&2; }

require_cmd() {
  if ! command -v "$1" &>/dev/null; then
    err "$1 is not installed. $2"
    exit 1
  fi
}

# ---------- pre-flight checks -----------------------------------------------

require_cmd appwrite "Install it with: npm install -g appwrite-cli"
require_cmd jq       "Install it with: brew install jq (macOS) or apt install jq (Linux)"

# ---------- defaults ---------------------------------------------------------

DEFAULT_ENDPOINT="https://cloud.appwrite.io/v1"
DEFAULT_PROJECT_NAME="appwrite-rust-playground"
DEFAULT_PROJECT_ID="appwrite-rust-playground"
API_KEY_NAME="rust-playground-key"
SELF_SIGNED="false"

# All scopes the playground demos need
API_KEY_SCOPES=(
  # Health
  health.read
  # Databases
  databases.read databases.write
  collections.read collections.write
  attributes.read attributes.write
  indexes.read indexes.write
  documents.read documents.write
  # Storage
  buckets.read buckets.write
  files.read files.write
  # Users
  users.read users.write
  # Functions
  functions.read functions.write
  executions.read executions.write
)

# ---------- gather inputs ----------------------------------------------------

read -rp "Appwrite endpoint [$DEFAULT_ENDPOINT]: " input_endpoint
ENDPOINT="${input_endpoint:-$DEFAULT_ENDPOINT}"

read -rp "Organization ID (find it in your Appwrite console): " ORG_ID
if [ -z "$ORG_ID" ]; then
  err "Organization ID is required."
  exit 1
fi

read -rp "Project name [$DEFAULT_PROJECT_NAME]: " input_name
PROJECT_NAME="${input_name:-$DEFAULT_PROJECT_NAME}"

read -rp "Project ID [$DEFAULT_PROJECT_ID]: " input_id
PROJECT_ID="${input_id:-$DEFAULT_PROJECT_ID}"

read -rp "Use self-signed certificates? (true/false) [$SELF_SIGNED]: " input_ss
SELF_SIGNED="${input_ss:-$SELF_SIGNED}"

# ---------- create project ---------------------------------------------------

info "Creating project '$PROJECT_NAME'…"
(cd "$SCRIPT_DIR" && appwrite init project \
  --organization-id "$ORG_ID" \
  --project-id "$PROJECT_ID" \
  --project-name "$PROJECT_NAME")

# Verify the config file was written
APPWRITE_JSON="$SCRIPT_DIR/appwrite.config.json"
if [ ! -f "$APPWRITE_JSON" ]; then
  err "appwrite.config.json not found — project init may have failed."
  exit 1
fi

ok "Project ready: $PROJECT_ID (endpoint: $ENDPOINT)"

# ---------- create API key ---------------------------------------------------

info "Creating API key '$API_KEY_NAME' with all playground scopes…"
KEY_JSON=$(appwrite projects create-key \
  --project-id "$PROJECT_ID" \
  --name "$API_KEY_NAME" \
  --scopes "${API_KEY_SCOPES[@]}" \
  --json) || {
  err "Failed to create API key."
  exit 1
}

API_KEY=$(echo "$KEY_JSON" | jq -r '.secret')

if [ -z "$API_KEY" ] || [ "$API_KEY" = "null" ]; then
  err "API key was created but the secret could not be read."
  exit 1
fi

ok "API key created."

# ---------- write .env -------------------------------------------------------

ENV_FILE="$SCRIPT_DIR/.env"

cat > "$ENV_FILE" <<EOF
APPWRITE_ENDPOINT=$ENDPOINT
APPWRITE_PROJECT_ID=$PROJECT_ID
APPWRITE_API_KEY=$API_KEY
APPWRITE_SELF_SIGNED=$SELF_SIGNED
APPWRITE_SAMPLE_FILE=resources/sample-upload.txt
APPWRITE_FUNCTION_SOURCE_DIR=resources/functions/hello-node
EOF

ok "Credentials written to $ENV_FILE"

# ---------- done -------------------------------------------------------------

echo ""
echo "You're all set! Run the playground with:"
echo ""
echo "  set -a && source .env && set +a"
echo "  cargo run -- all"
echo ""
