#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGDATA="${PGDATA:-$ROOT/.dev-pgdata}"
PGHOST="${PGHOST:-$ROOT/.dev-pgsocket}"
DB_NAME="${PLINTH_DEV_DB_NAME:-plinth}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1. Run this from nix develop." >&2
    exit 1
  fi
}

ensure_tools() {
  need_cmd initdb
  need_cmd pg_ctl
  need_cmd createdb
  need_cmd psql
}

is_running() {
  [ -d "$PGDATA" ] && pg_ctl -D "$PGDATA" status >/dev/null 2>&1
}

start_db() {
  ensure_tools

  mkdir -p "$PGHOST"

  if [ ! -d "$PGDATA" ]; then
    initdb -D "$PGDATA" --auth=trust --no-locale --encoding=UTF8
  fi

  if ! is_running; then
    pg_ctl -D "$PGDATA" -l "$PGDATA/postgres.log" -o "-k $PGHOST -h ''" start
  fi

  createdb -h "$PGHOST" "$DB_NAME" >/dev/null 2>&1 || true
  psql -h "$PGHOST" -d "$DB_NAME" -v ON_ERROR_STOP=1 -c "CREATE EXTENSION IF NOT EXISTS vector"
}

stop_db() {
  ensure_tools

  if is_running; then
    pg_ctl -D "$PGDATA" stop
  fi
}

reset_db() {
  stop_db
  rm -rf "$PGDATA" "$PGHOST"
  start_db
}

case "${1:-}" in
  start)
    start_db
    ;;
  stop)
    stop_db
    ;;
  reset)
    reset_db
    ;;
  *)
    echo "Usage: $0 start|stop|reset" >&2
    exit 2
    ;;
esac
