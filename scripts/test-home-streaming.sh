#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1. Run this from nix develop." >&2
    exit 1
  fi
}

need_cmd cargo
need_cmd curl
need_cmd perl
need_cmd rg

export PGDATA="/tmp/plinth-home-streaming-pgdata"
export PGHOST="/tmp/plinth-home-streaming-pgsock"
export PLINTH_DEV_DB_NAME="plinth_home_streaming"
export DATABASE_URL="postgres://$(id -un)@localhost/${PLINTH_DEV_DB_NAME}?host=${PGHOST}"
export PLINTH_DATABASE_URL="$DATABASE_URL"
export PLINTH_API_KEY="${PLINTH_API_KEY:-phase03_streaming_key}"
export PLINTH_SITE_ADDR="${PLINTH_SITE_ADDR:-127.0.0.1:3220}"
export DIOXUS_PUBLIC_PATH="${DIOXUS_PUBLIC_PATH:-target/site}"
export PLINTH_RENDER_CACHE_DIR="${PLINTH_RENDER_CACHE_DIR:-/tmp/plinth-home-streaming-render-cache}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export RUST_LOG="${RUST_LOG:-warn}"
export PLINTH_TEST_ACTIVITY_DELAY_MS="${PLINTH_TEST_ACTIVITY_DELAY_MS:-1500}"

rm -rf "$PGDATA" "$PGHOST" "$PLINTH_RENDER_CACHE_DIR"
scripts/dev-db.sh start >/tmp/plinth-home-streaming-db.log 2>&1

server_pid=""
cleanup() {
  if [ -n "$server_pid" ]; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
  scripts/dev-db.sh stop >/tmp/plinth-home-streaming-db-stop.log 2>&1 || true
}
trap cleanup EXIT

cargo build --locked --package plinth-web --bin plinth-web --no-default-features --features server,brick-blog,brick-portfolio,brick-todo,brick-activity >/tmp/plinth-home-streaming-build.log 2>&1
target/debug/plinth-web >/tmp/plinth-home-streaming-server.log 2>&1 &
server_pid=$!

for i in $(seq 1 240); do
  if curl -fsS "http://${PLINTH_SITE_ADDR}/api/health" >/tmp/plinth-home-streaming-health.json 2>/tmp/plinth-home-streaming-health.err; then
    break
  fi
  if ! kill -0 "$server_pid" >/dev/null 2>&1; then
    echo "Server exited before health check." >&2
    cat /tmp/plinth-home-streaming-server.log >&2
    exit 1
  fi
  sleep 0.25
  if [ "$i" -eq 240 ]; then
    echo "Server did not become healthy." >&2
    cat /tmp/plinth-home-streaming-server.log >&2
    exit 1
  fi
done

AUTH=(-H "Authorization: Bearer ${PLINTH_API_KEY}" -H "Content-Type: application/json")

curl -fsS -X PUT "http://${PLINTH_SITE_ADDR}/api/admin/content/home-intro" "${AUTH[@]}" --data-binary @- >/tmp/plinth-home-streaming-site.json <<'JSON'
{"title":"Phase 03 Intro","content":"Phase 03 intro content","html_content":"<p>Phase 03 Intro</p>"}
JSON

curl -fsS -X POST "http://${PLINTH_SITE_ADDR}/api/admin/articles" "${AUTH[@]}" --data-binary @- >/tmp/plinth-home-streaming-blog.json <<'JSON'
{"title":"Phase 03 Blog Stream","slug":"phase-03-blog-stream","description":"Blog streaming description","content":"# Phase 03 Blog Stream\n\nBody for streaming test.","author":"Streaming Tester","tags":["streaming"],"published":true,"featured":false}
JSON

curl -fsS -X POST "http://${PLINTH_SITE_ADDR}/api/admin/portfolio" "${AUTH[@]}" --data-binary @- >/tmp/plinth-home-streaming-portfolio.json <<'JSON'
{"slug":"phase-03-project-stream","title":"Phase 03 Project Stream","description":"Portfolio streaming description","content":"# Phase 03 Project Stream","tech_stack":["Rust","Dioxus"],"date":"2026-06-03T00:00:00Z","featured":false,"order":0,"content_format":"markdown"}
JSON

curl -fsS -X POST "http://${PLINTH_SITE_ADDR}/api/admin/activity" "${AUTH[@]}" --data-binary @- >/tmp/plinth-home-streaming-activity.json <<'JSON'
{"forge":"github","repo_owner":"phase","repo_name":"streaming","kind":"pr","number":303,"url":"https://github.com/phase/streaming/pull/303","title":"Phase 03 Slow Activity","body":"Activity body","state":"merged","created_at":"2026-06-03T00:00:00Z","merged_at":"2026-06-03T00:00:00Z","impact":9,"labels":["streaming"],"featured":true,"published":true}
JSON

export STREAM_HOST="${PLINTH_SITE_ADDR%:*}"
export STREAM_PORT="${PLINTH_SITE_ADDR##*:}"
perl <<'PL'
use strict;
use warnings;
use IO::Socket::INET;
use Time::HiRes qw(time);

my $host = $ENV{"STREAM_HOST"};
my $port = $ENV{"STREAM_PORT"};
my %markers = (
  shell => "<body>",
  intro => "Phase 03 Intro",
  blog => "Phase 03 Blog Stream",
  portfolio => "Phase 03 Project Stream",
  activity => "Phase 03 Slow Activity",
);
my %seen;
my $buf = "";
my $start = time();

my $sock = IO::Socket::INET->new(
  PeerHost => $host,
  PeerPort => $port,
  Proto => "tcp",
  Timeout => 10,
) or die "connect failed: $!";

print $sock
  "GET / HTTP/1.1\r\n",
  "Host: $host:$port\r\n",
  "Accept: text/html\r\n",
  "Accept-Encoding: identity\r\n",
  "Connection: close\r\n",
  "\r\n";

while (1) {
  my $chunk = "";
  my $n = sysread($sock, $chunk, 4096);
  last if !defined($n) || $n == 0;
  $buf .= $chunk;
  my $now = time() - $start;
  for my $name (keys %markers) {
    $seen{$name} = $now if !exists($seen{$name}) && index($buf, $markers{$name}) >= 0;
  }
  last if scalar(keys %seen) == scalar(keys %markers);
}

my @missing = grep { !exists($seen{$_}) } keys %markers;
die "missing streamed markers: @missing\n" if @missing;

die "shell did not arrive before activity\n" unless $seen{shell} < $seen{activity};
die "intro did not arrive before delayed activity\n" unless $seen{intro} < $seen{activity};
die "blog was blocked by delayed activity\n" unless $seen{blog} < $seen{activity};
die "portfolio was blocked by delayed activity\n" unless $seen{portfolio} < $seen{activity};

my $delay_seconds = $ENV{"PLINTH_TEST_ACTIVITY_DELAY_MS"} / 1000.0;
die "activity marker arrived too early for injected delay\n"
  if $seen{activity} < $delay_seconds * 0.75;
die "blog marker arrived too late; likely blocked by activity\n"
  if $seen{blog} > $delay_seconds * 0.75;
die "portfolio marker arrived too late; likely blocked by activity\n"
  if $seen{portfolio} > $delay_seconds * 0.75;

printf "home streaming marker timings: shell=%.3f intro=%.3f blog=%.3f portfolio=%.3f activity=%.3f\n",
  $seen{shell}, $seen{intro}, $seen{blog}, $seen{portfolio}, $seen{activity};
PL

curl -fsS -H "Accept-Encoding: identity" "http://${PLINTH_SITE_ADDR}/" >/tmp/plinth-home-streaming-final.html
rg -F "Phase 03 Intro" /tmp/plinth-home-streaming-final.html >/dev/null
rg -F "Phase 03 Blog Stream" /tmp/plinth-home-streaming-final.html >/dev/null
rg -F "Phase 03 Project Stream" /tmp/plinth-home-streaming-final.html >/dev/null
rg -F "Phase 03 Slow Activity" /tmp/plinth-home-streaming-final.html >/dev/null
! rg -F "Could not load" /tmp/plinth-home-streaming-final.html >/dev/null

echo "home streaming smoke test passed"
