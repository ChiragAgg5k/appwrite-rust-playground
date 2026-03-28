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

DIM='\033[2m'
RESET='\033[0m'
BOLD='\033[1m'
GREEN='\033[1;32m'
BLUE='\033[1;34m'
RED='\033[1;31m'
CYAN='\033[1;36m'

info()  { printf "${BLUE}▸ %s${RESET}\n" "$*"; }
ok()    { printf "${GREEN}✔ %s${RESET}\n" "$*"; }
err()   { printf "${RED}✘ %s${RESET}\n" "$*" >&2; }
dim()   { printf "${DIM}%s${RESET}" "$*"; }

banner() {
  echo ""
  printf "${BOLD}${CYAN}"
  echo "  ┌──────────────────────────────────────┐"
  echo "  │   Appwrite Rust Playground · Setup    │"
  echo "  └──────────────────────────────────────┘"
  printf "${RESET}"
  echo ""
}

require_cmd() {
  if ! command -v "$1" &>/dev/null; then
    err "$1 is not installed. $2"
    exit 1
  fi
}

# ---------- pre-flight checks -----------------------------------------------

require_cmd appwrite "Install it with: npm install -g appwrite-cli"
require_cmd jq       "Install it with: brew install jq (macOS) or apt install jq (Linux)"

banner

# ---------- defaults ---------------------------------------------------------

ENDPOINT="https://cloud.appwrite.io/v1"
PROJECT_NAME="appwrite-rust-playground"
RANDOM_SUFFIX=$(head -c 4 /dev/urandom | xxd -p)
PROJECT_ID="rust-playground-${RANDOM_SUFFIX}"
API_KEY_NAME="rust-playground-key"
SELF_SIGNED="false"
ORG_ID=""

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
  execution.read execution.write
)

# ---------- usage -----------------------------------------------------------

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --org-id ID              Organization ID (skips auto-detection)
  --project-name NAME      Project name          [default: $PROJECT_NAME]
  --project-id ID          Project ID            [default: $PROJECT_ID]
  --endpoint URL           Appwrite endpoint     [default: $ENDPOINT]
  --self-signed true|false Use self-signed certs [default: $SELF_SIGNED]
  -h, --help               Show this help message

Examples:
  # Use defaults — auto-detects org from your Appwrite account
  ./setup.sh

  # Fully customised
  ./setup.sh --org-id 64a1b2c3d4e5f --project-name my-app --project-id my-app-id
EOF
  exit 0
}

# ---------- parse CLI args ---------------------------------------------------

while [[ $# -gt 0 ]]; do
  case "$1" in
    --org-id)        ORG_ID="$2";        shift 2 ;;
    --project-name)  PROJECT_NAME="$2";  shift 2 ;;
    --project-id)    PROJECT_ID="$2";    shift 2 ;;
    --endpoint)      ENDPOINT="$2";      shift 2 ;;
    --self-signed)   SELF_SIGNED="$2";   shift 2 ;;
    -h|--help)       usage ;;
    *)               err "Unknown option: $1"; usage ;;
  esac
done

# ---------- resolve organization ID -----------------------------------------

if [ -z "$ORG_ID" ]; then
  info "Fetching your Appwrite organizations…"
  ORGS_JSON=$(appwrite organizations list --json 2>/dev/null) || {
    err "Failed to list organizations. Make sure you are logged in (appwrite login)."
    exit 1
  }

  ORG_COUNT=$(echo "$ORGS_JSON" | jq '.total')
  if [ "$ORG_COUNT" -eq 0 ]; then
    err "No organizations found in your Appwrite account. Create one first."
    exit 1
  elif [ "$ORG_COUNT" -eq 1 ]; then
    ORG_ID=$(echo "$ORGS_JSON" | jq -r '.organizations[0].$id // .teams[0].$id')
    ORG_NAME=$(echo "$ORGS_JSON" | jq -r '.organizations[0].name // .teams[0].name')
    ok "Auto-selected organization: $ORG_NAME $(dim "($ORG_ID)")"
  else
    echo ""
    echo "  Multiple organizations found:"
    echo ""
    echo "$ORGS_JSON" | jq -r '(.organizations // .teams) | to_entries[] | "    \(.key + 1)) \(.value.name) \u001b[2m(\(.value."$id"))\u001b[0m"'
    echo ""
    read -rp "  Select organization [1]: " ORG_CHOICE
    ORG_CHOICE="${ORG_CHOICE:-1}"
    ORG_IDX=$((ORG_CHOICE - 1))
    ORG_ID=$(echo "$ORGS_JSON" | jq -r "(.organizations // .teams)[$ORG_IDX].\"\$id\"")
    ORG_NAME=$(echo "$ORGS_JSON" | jq -r "(.organizations // .teams)[$ORG_IDX].name")
    if [ -z "$ORG_ID" ] || [ "$ORG_ID" = "null" ]; then
      err "Invalid selection."
      exit 1
    fi
    ok "Selected organization: $ORG_NAME $(dim "($ORG_ID)")"
  fi
fi

# ---------- create project ---------------------------------------------------

info "Creating project '$PROJECT_NAME'…"
(cd "$SCRIPT_DIR" && appwrite init project \
  --organization-id "$ORG_ID" \
  --project-id "$PROJECT_ID" \
  --project-name "$PROJECT_NAME") > /dev/null

# Verify the config file was written
APPWRITE_JSON="$SCRIPT_DIR/appwrite.config.json"
if [ ! -f "$APPWRITE_JSON" ]; then
  err "appwrite.config.json not found — project init may have failed."
  exit 1
fi

ok "Project created $(dim "($PROJECT_ID)")"

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

ok "Credentials written to .env"

# ---------- done -------------------------------------------------------------

echo ""
printf "${GREEN}${BOLD}  All set!${RESET} Run the playground with:\n"
echo ""
printf "    ${BOLD}cargo run -- all${RESET}\n"
echo ""
