# Progress Pages

`plinth-progress` turns a small `progress.toml` in your repository into one
self-contained `index.html`: a status page for collaborators that shows the
current release in full and earlier releases as collapsed history. A bump guard
makes sure the page is updated every time the project's version changes.

The source format, validation rules and diagnostics are documented in
[`crates/progress/README.md`](https://github.com/caniko/plinth/blob/trunk/crates/progress/README.md),
with a complete example in `crates/progress/examples/progress.toml` and a JSON
Schema in `crates/progress/progress.schema.json`. Start your file with
`#:schema <path-to>/progress.schema.json` to get completion and inline errors in
editors that support it.

## Commands

| Command | Does |
|---------|------|
| `plinth-progress check --version <V> <SOURCE>` | Validates the whole file, then fails unless it has an entry for exactly `<V>`. Silent on success. |
| `plinth-progress build <SOURCE> --out <FILE> [--config <FILE>]` | Validates, then writes the page. Never touches `--out` on failure. |
| `plinth-progress draft --version <V> [--since <REF>] [--date <YYYY-MM-DD>] <SOURCE>` | Prints an entry skeleton for `<V>` with the commits since the last tag. Never edits files. |

Exit codes: `0` success, `1` validation, guard or I/O failure, `2` usage error.
Errors read `error: <file>:<line>:<column>: <field-path>: <message>`.

## The page

The page is a single file with inline CSS. It has no scripts, stylesheet links,
images or fonts to fetch, carries `noindex,nofollow`, and sets a
`Content-Security-Policy` meta tag of `default-src 'none'; style-src 'unsafe-inline'`.
Text fields accept plain text with `` `code` `` spans and `[label](https://...)`
links (`http`, `https`, `/` and `#` only); everything else is escaped.

`--config` points at your existing `plinth-project.toml`. Only its `[theme]`
table is read, so the page matches your project site. Two theme settings are
treated differently:

- `font_url` is dropped with a warning, because the page fetches nothing;
  `font_family` stays and falls back to system fonts.
- `broker_css` is refused. Set explicit `[theme]` colours instead.

Colour values must be plain CSS colours (`#hex`, names, `rgb()`, `hsl()`,
`oklch()`, `color-mix()` and similar). Anything else fails the build.

The page is not access control. If it is private, serve it behind your own
authentication and keep it out of public static assets.

## Nix

```nix
progressPage = plinth.lib.${system}.mkProgressPage {
  src = ./progress.toml;                    # required
  config = ./website/plinth-project.toml;   # optional, [theme] only
  version = cargoToml.workspace.package.version;  # optional bump guard
  pname = "my-project-progress";            # optional
};
# => $out/index.html
```

With `version` set, the derivation runs `plinth-progress check` first, so a
version bump without a matching entry fails the build. The CLI is also
available as `plinth.packages.${system}.plinth-progress`.

## Wiring the bump guard

The guard only helps if it runs where versions change. Call it from both your
bump command and your qualification (CI) path, reading the version from the
file that owns it:

```bash
version=$(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[] | select(.name == "my-crate") | .version')
plinth-progress check --version "$version" progress.toml
```

Passing `version` to `mkProgressPage` covers the Nix build path with no extra
step.

## Updating on every bump (LLM workflow)

Give an agent this prompt when the version changes. It works for a human too.

```text
The project version is now <NEW_VERSION>. Update progress.toml so it has a
complete, truthful entry for it.

1. Run: plinth-progress draft --version <NEW_VERSION> progress.toml
   It prints a skeleton plus the commits since the last tag. If it warns that
   no tag was found, or the listed range looks wrong, rerun with
   --since <previous release tag> and say which ref you used.
2. Paste the skeleton at the top of the [[entries]] list, above the previous
   release. Do not edit or reorder older entries.
3. Fill every field from the listed commits, the CHANGELOG section for
   <NEW_VERSION> and the previous entry's `next` and `risks`:
   - summary: one or two sentences on what this release means for readers.
   - done: shipped items, one per line, user-facing wording.
   - next / risks: carry forward what is still open; drop what is resolved.
   - milestones: update statuses (done, in_progress, blocked, planned,
     dropped); add a label only when it adds a date or a reason.
   - sections: only for content that does not fit the lists.
   Use only facts you can point to in the commits, changelog or previous entry.
   If something is unclear, leave it out and ask; never invent dates, numbers
   or outcomes.
4. Run: plinth-progress check --version <NEW_VERSION> progress.toml
   Fix every reported error (file:line:column: field path) and rerun until it
   passes silently.
5. Run the project's page build (mkProgressPage or plinth-progress build) and
   confirm it succeeds.
6. Delete the draft's `#` comment lines before committing.
```

Checklist for reviewing the result:

- [ ] The new entry is first and its `version` equals the bumped version.
- [ ] `summary` is filled; no field still holds draft placeholder comments.
- [ ] Every `done` item traces to a commit or changelog line.
- [ ] Open items from the previous entry are carried forward or explicitly closed.
- [ ] `plinth-progress check --version <V> progress.toml` passes.
- [ ] The page builds, and nothing private landed in a public asset.
