# Project landing design

`project-landing.fig` is the editable OpenPencil reference for Plinth's guided
static renderer. Open it with `openpencil-desktop design/project-landing.fig`.
It contains R01, with desktop (1440 × 1280) and mobile (390 × 1380) artboards.
The example content belongs to Pink Raven; Plinth owns the reusable composition.

The document was extracted from Pink Raven's repaired native document at commit
`13dc4e11b5ca3e70bdf5e8d96efd10829cba2463`. `migration.json` records both saved
document hashes and the preservation check. Its `snapshot_revision` points to
the preserved extraction before subsequent illustration edits. All 229 R01
page-tree nodes in that snapshot retain
their geometry, text, vector networks, styling, variables, and original import
plugin data. OpenPencil renumbered serialization-local node IDs during extraction;
ordered tree structure and every other node property compare equal. Variables,
collections, images, enabled libraries, and color-space metadata were preserved.

Pink Raven's original monolithic `.op` import and historical migration receipt
remain available in that repository. Its current native document contains only
application references, with R01 removed through OpenPencil.

The extracted document's overlap/overflow audit found zero unintended findings.
The 239 explained intersections are illustration artwork. The static renderer
uses fluid widths and content-driven heights; the font and illustration assets
are supplied by the consuming site's configuration.

The current document replaces the blank hero block with a labeled illustrative
civic diagram and a room-function legend. The example is explicitly identified
as a diagram rather than a real library record. Metadata uses readable 12–14px
type on desktop; the mobile illustration has its own composition. The current
saved-file audit reports zero unintended findings and 14 explained illustration
intersections. Native 2× PNG exports supply the consuming site's preview assets.
Saved-file preview node IDs are `0:27` (desktop) and `0:153` (mobile); regenerate
these IDs after future document saves.

```sh
openpencil info design/project-landing.fig
openpencil export design/project-landing.fig --page 'R01 · Plinth landing · /' \
  --font-policy strict --output target/project-landing.png
cargo test -p plinth-project
```
