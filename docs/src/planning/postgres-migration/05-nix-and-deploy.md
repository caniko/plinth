# Phase 05 — Update NixOS module, flake, and dev shell for Postgres

> **Recommended Codex model: GPT 5.5 medium**
>
> Nix module work: declarative service config, dev-shell dependency wiring, ensuring pgvector is available in the chosen Postgres package, and updating systemd `wants=`/`after=` ordering so Plinth starts after Postgres. Medium tier handles `services.postgresql` configuration competently. Low tier risks missing the `extraPlugins` syntax for pgvector or forgetting the `ensureDatabases`/`ensureUsers` boilerplate; high is overkill.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo. Independent of Phases 03 and 04 (disjoint files); depends only on Phase 01 (config field rename) and Phase 02 (knowing what extensions are needed).

## Goal

A fresh `nixos-rebuild test` on a host with the Plinth NixOS module enabled brings up:

- `postgresql.service` with the `pgvector` extension loaded.
- A `plinth` database and `plinth` role owned by the Plinth service user.
- The `plinth` service unit ordered `After=postgresql.service` with `Wants=` set, and configured with the correct `DATABASE_URL` env var pointing at the local socket.
- `nix develop` (dev shell) provides `postgresql` on `PATH`, `sqlx-cli`, and a `pg_ctl`-based local DB lifecycle script (`scripts/dev-db.sh start|stop|reset`).

## Why this matters now

Phase 01 made Postgres mandatory at runtime; Phase 02 made schema-and-extension assumptions concrete. Until this phase lands, nobody can actually *run* Plinth — `cargo run` will fail to connect, and a NixOS deployment will be broken. The integration tests (Phase 06) also need a local Postgres available via `nix develop`, so this is a hard prerequisite for that phase's verification.

## Out of scope

- Production Postgres tuning (`shared_buffers`, etc.) — Plinth's scale doesn't need it; defaults are fine.
- Backup/restore strategy — separate concern.
- TLS to Postgres — using the Unix socket on the same host avoids the need.
- Migration data export from any pre-existing SurrealDB deployment — not in scope per Phase 02.

## Plan

1. **Pick Postgres version.** `postgresql_16` from current nixpkgs. Pin via the existing `flake.nix` `nixpkgs` input — don't add a new input.
2. **NixOS module updates** in `nix/module.nix` (or wherever the Plinth NixOS module lives — grep `services.plinth` to locate):
   ```nix
   services.postgresql = {
     enable = true;
     package = pkgs.postgresql_16;
     extensions = ps: [ ps.pgvector ];
     ensureDatabases = [ "plinth" ];
     ensureUsers = [{
       name = "plinth";
       ensureDBOwnership = true;
     }];
   };

   systemd.services.plinth = {
     after = [ "postgresql.service" ];
     wants = [ "postgresql.service" ];
     environment.DATABASE_URL = "postgres:///plinth?host=/run/postgresql";
     serviceConfig.User = "plinth";
   };
   ```
   Use peer authentication via the Unix socket — no password needed, no TLS, no `pg_hba.conf` edits.
3. **Dev shell** in `flake.nix`:
   ```nix
   devShells.default = pkgs.mkShell {
     packages = with pkgs; [
       postgresql_16
       sqlx-cli
       # …existing rust toolchain…
     ];
     shellHook = ''
       export PGDATA="$PWD/.dev-pgdata"
       export DATABASE_URL="postgres://localhost/plinth?host=$PWD/.dev-pgsocket"
       # ...
     '';
   };
   ```
4. **Dev DB script** `scripts/dev-db.sh`:
   - `start`: `initdb` if `.dev-pgdata` missing, then `pg_ctl start -o "-k $PWD/.dev-pgsocket"`, then `createdb plinth || true`, then `psql -c 'CREATE EXTENSION IF NOT EXISTS vector'`.
   - `stop`: `pg_ctl stop`.
   - `reset`: stop, `rm -rf .dev-pgdata .dev-pgsocket`, start.
   Add `.dev-pgdata` and `.dev-pgsocket` to `.gitignore`.
5. **Update `docs/book/src/deployment/nixos-module.md`** and `docs/book/src/development/setup.md` to reference the new flow. (Full doc rewrite is Phase 06; this phase just keeps the deploy doc not-actively-wrong.)

## Acceptance criteria

- [ ] `nix flake check` exits 0.
- [ ] `nix develop --command bash -c 'which psql && which sqlx'` prints both binaries.
- [ ] `nix develop --command ./scripts/dev-db.sh start` exits 0; subsequent `psql -l` lists `plinth`; `psql plinth -c 'SELECT extname FROM pg_extension'` includes `vector`.
- [ ] `nixos-rebuild test` (or `nix build .#nixosConfigurations.<test-host>.config.system.build.toplevel`) succeeds on at least one host that imports the Plinth module.
- [ ] `systemctl status plinth` on that test host shows the unit Active and the journal contains a "connected to postgres" line (or equivalent — at minimum, no DB-connection panic).
- [ ] `.dev-pgdata` and `.dev-pgsocket` are gitignored.

## Files likely touched

- `flake.nix` (dev shell, possibly module export)
- `nix/module.nix` or `nix/plinth-module.nix` (locate via `rg 'services.plinth'`)
- `scripts/dev-db.sh` (new)
- `.gitignore`
- `docs/book/src/deployment/nixos-module.md` (minimal update)
- `docs/book/src/development/setup.md` (minimal update)

## Pitfalls

- **`services.postgresql.extensions` vs `extraPlugins`.** The attribute name has changed across nixpkgs versions. Recent nixpkgs uses `extensions = ps: [ ps.pgvector ];`; older used `extraPlugins = with pkgs.postgresql_16.pkgs; [ pgvector ];`. Check the nixpkgs revision pinned in `flake.lock` and use the matching attribute. Test with `nix-instantiate --eval -E 'with import <nixpkgs> {}; postgresql.pkgs.pgvector or postgresql.extensions or null'` if unsure.
- **`ensureDBOwnership = true`** is required for the `plinth` role to own the `plinth` DB; without it, migrations will fail with permission errors on `CREATE EXTENSION` (which requires superuser unless preloaded). Alternative: preload extension via `services.postgresql.extraPlugins` so the extension already exists when migrations run.
- **`CREATE EXTENSION` requires superuser** in vanilla Postgres. NixOS works around this by letting the postgres init script create extensions; rely on `ensureDatabases` + a one-shot systemd unit that runs `CREATE EXTENSION IF NOT EXISTS vector` as the `postgres` user. Or accept that the *first* `sqlx migrate run` must be done manually as the postgres user. Document whichever you pick.
- **Socket path length.** Unix socket paths longer than 107 chars silently fail on Linux. Don't put `.dev-pgsocket` inside a deeply nested workspace path; if you must, symlink to a short path under `/tmp`.
- **`pg_ctl` and Nix.** `pg_ctl` needs `PGDATA` set; the dev shell exports it. If a user runs `pg_ctl` from a non-`nix develop` shell, it will fail confusingly. Note in `setup.md`.

## Reference

- Audit transcript: chat session 2026-05-19.
- nixpkgs pgvector: <https://search.nixos.org/packages?show=postgresql16Packages.pgvector>.
- NixOS option for extensions: `services.postgresql.extensions`.
- Prev: parallel with [03-query-rewrite.md](./03-query-rewrite.md) and [04-vector-search-pgvector.md](./04-vector-search-pgvector.md). Next: [06-tests-and-docs.md](./06-tests-and-docs.md).
