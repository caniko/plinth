# plinth-progress

Validates a project progress source and guards an exact release version in it.

## Source format

One UTF-8 TOML document that lists releases newest first:

```toml
#:schema ./progress.schema.json
title = "Example project progress"

[[entries]]
version = "1.2.0" # canonical SemVer, no leading v
date = "2026-11-17" # valid YYYY-MM-DD calendar date
summary = "The first release is ready."
done = ["Published the core feature."]
next = ["Measure adoption."]
risks = ["Migration needs review."]

[[entries.milestones]]
name = "Core release"
status = "done" # done | in_progress | blocked | planned | dropped
label = "Shipped" # optional display-only text

[[entries.sections]]
heading = "Decisions"
body = "Keep the first release small.\nReview the next phase later."
```

Every field above is required except `label`. Arrays may be empty except
`entries`. Text is plain text with `code` and [links](https://example.com)
(rendering arrives with the page builder). Unknown keys, blank text, malformed
SemVer, impossible calendar dates and unordered entries or dates are rejected.

Rules that the JSON Schema cannot express are enforced at runtime: entry
versions are unique as exact strings and strictly descending by SemVer
precedence (build metadata is kept but ignored for precedence, so `1.0.0+a`
and `1.0.0+b` are equal-precedence and therefore an error), and dates are real
Gregorian dates that never increase.

## Check a version

```console
plinth-progress check --version 1.2.0 progress.toml
```

Exit codes: `0` when the version is present, `1` for input, validation or
guard errors, `2` for command-line usage errors. Nothing is printed on success.

Diagnostics use `error: <file>:<line>:<column>: <field-path>: <message>`
(one-based, Unicode-character columns), for example:

```console
error: progress.toml:4:1: entries[0]: missing required field "risks"
```

`--version` must itself be canonical SemVer: `v1.2.0` and `1.2` are rejected
before the membership test. The guard proves that a version is listed, not that
it is the newest entry.
