#!/usr/bin/env bash
set -euo pipefail

# Civico local development launcher.
# Starts PostgreSQL, runs migrations, then launches API, CMS, and Player.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"

# Ensure cargo-installed binaries (e.g. sqlx-cli) are available.
export PATH="${HOME}/.cargo/bin:${PATH}"

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

log() {
  echo "[dev.sh] $*"
}

warn() {
  echo "[dev.sh] WARNING: $*" >&2
}

err() {
  echo "[dev.sh] ERROR: $*" >&2
  exit 1
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Detect container runtime: prefer podman, fall back to docker.
detect_container_runtime() {
  if command_exists podman; then
    echo "podman"
  elif command_exists docker; then
    echo "docker"
  else
    err "Neither podman nor docker found. Please install one of them."
  fi
}

# Check whether a TCP port is already listening.
is_port_in_use() {
  local port="$1"
  if command_exists ss; then
    ss -tln 2>/dev/null | grep -qE ":${port}[[:space:]]"
  elif command_exists netstat; then
    netstat -tln 2>/dev/null | grep -qE ":${port}[[:space:]]"
  else
    # Fallback: assume free if we cannot check.
    return 1
  fi
}

# -----------------------------------------------------------------------------
# Sanity checks
# -----------------------------------------------------------------------------

for cmd in cargo npm; do
  command_exists "$cmd" || err "Missing required tool: $cmd"
done

# sqlx-cli is needed for migrations.
command_exists sqlx || err "Missing sqlx-cli. Install with: cargo install sqlx-cli --no-default-features --features native-tls,postgres"

RUNTIME="$(detect_container_runtime)"
log "Using container runtime: $RUNTIME"

# -----------------------------------------------------------------------------
# Environment file
# -----------------------------------------------------------------------------

if [[ ! -f "$ENV_FILE" ]]; then
  if [[ -f "${ENV_FILE}.example" ]]; then
    log "Creating .env from .env.example"
    cp "${ENV_FILE}.example" "$ENV_FILE"
    warn "Generated .env with default values. Change JWT_SECRET for real use."
  else
    err ".env.example not found. Cannot create .env"
  fi
fi

# Load environment variables into the current shell.
set -a
# shellcheck source=/dev/null
source "$ENV_FILE"
set +a

# Ensure required variables are present.
: "${DATABASE_URL:?DATABASE_URL is not set in .env}"
: "${JWT_SECRET:?JWT_SECRET is not set in .env}"

API_BIND="${API_BIND:-127.0.0.1:8080}"
API_PORT="${API_BIND##*:}"
CMS_PORT=8082
PLAYER_PORT=8081

# Derive the API base URL used by the frontend dev servers.
# Defaults to http:// + API_BIND so a custom API port is automatically picked up.
API_BASE_URL="${VITE_API_URL:-http://${API_BIND}}"
PLAYER_URL="${VITE_PLAYER_URL:-http://127.0.0.1:${PLAYER_PORT}}"

for port in "$API_PORT" "$CMS_PORT" "$PLAYER_PORT"; do
  if is_port_in_use "$port"; then
    err "Port ${port} is already in use. Stop the conflicting service first or change it in .env."
  fi
done

# -----------------------------------------------------------------------------
# Start PostgreSQL
# -----------------------------------------------------------------------------

start_postgres() {
  log "Starting PostgreSQL with $RUNTIME compose..."
  "$RUNTIME" compose -f "${SCRIPT_DIR}/compose.yaml" up -d postgres
}

wait_for_postgres() {
  log "Waiting for PostgreSQL to be ready..."
  local attempts=0
  local max_attempts=30
  while [[ $attempts -lt $max_attempts ]]; do
    if "$RUNTIME" exec civico-postgres pg_isready -U "${POSTGRES_USER:-civico}" -d "${POSTGRES_DB:-civico}" >/dev/null 2>&1; then
      log "PostgreSQL is ready."
      return 0
    fi
    attempts=$((attempts + 1))
    sleep 1
  done
  err "PostgreSQL did not become ready within ${max_attempts}s."
}

# -----------------------------------------------------------------------------
# Database migrations
# -----------------------------------------------------------------------------

run_migrations() {
  log "Running database migrations..."
  cd "${SCRIPT_DIR}/crates/api"
  cargo sqlx migrate run
  cd "$SCRIPT_DIR"
}

# -----------------------------------------------------------------------------
# Process management
# -----------------------------------------------------------------------------

PIDS=()

start_api() {
  log "Starting API on ${API_BIND:-127.0.0.1:8080}..."
  cargo run -p civico-api &
  PIDS+=("$!")
}

start_cms() {
  log "Starting CMS web dev server on http://127.0.0.1:${CMS_PORT}..."
  log "  (pointing to API at ${API_BASE_URL})"
  cd "${SCRIPT_DIR}/apps/cms-web"
  VITE_API_URL="${API_BASE_URL}" VITE_PLAYER_URL="${PLAYER_URL}" npm run dev -- --host 127.0.0.1 &
  PIDS+=("$!")
  cd "$SCRIPT_DIR"
}

start_player() {
  log "Starting Player web dev server on http://127.0.0.1:${PLAYER_PORT}..."
  cd "${SCRIPT_DIR}/apps/player-web"
  npm run dev -- --host 127.0.0.1 &
  PIDS+=("$!")
  cd "$SCRIPT_DIR"
}

shutdown() {
  log "Shutting down services..."
  for pid in "${PIDS[@]:-}"; do
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done
  wait >/dev/null 2>&1 || true
  log "Done."
}

trap shutdown INT TERM EXIT

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------

start_postgres
wait_for_postgres
run_migrations

start_api
start_cms
start_player

log ""
log "All services starting."
log "  API:    ${API_BASE_URL}"
log "  CMS:    http://127.0.0.1:${CMS_PORT}"
log "  Player: ${PLAYER_URL}"
log ""
log "Press Ctrl+C to stop everything."

# Wait until any background job exits.
wait
