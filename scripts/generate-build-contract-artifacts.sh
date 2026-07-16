#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

command -v pklx >/dev/null || {
  echo "pklx is required; run: nix develop -c $0" >&2
  exit 1
}
command -v pkl >/dev/null || {
  echo "pkl is required; run: nix develop -c $0" >&2
  exit 1
}

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

pklx eval website/portfolio.pkl > "$tmpdir/portfolio.generated.nix"
{
  printf '%s\n' '# Generated from website/portfolio.pkl with pklx eval.'
  cat "$tmpdir/portfolio.generated.nix"
} > website/portfolio.generated.nix

pkl eval -f json pkl/VisualAudit.fixture.pkl > pkl/VisualAudit.fixture.json

echo "Generated website/portfolio.generated.nix and pkl/VisualAudit.fixture.json"
