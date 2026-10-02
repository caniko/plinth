# Project landing design

`project-landing.fig` is the editable OpenPencil reference for Plinth's guided
static renderer. Open it with `openpencil-desktop design/project-landing.fig`.
It contains R01, with desktop (1440 × 1280) and mobile (390 × 1380) artboards.
The example content belongs to Pink Raven; Plinth owns the reusable composition.

The document was extracted from Pink Raven's repaired native document at commit
`13dc4e11b5ca3e70bdf5e8d96efd10829cba2463`. `migration.json` records both saved
document hashes and the preservation check. All 229 R01 page-tree nodes retain
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

```sh
openpencil info design/project-landing.fig
openpencil export design/project-landing.fig --page 'R01 · Plinth landing · /' \
  --font-policy strict --output target/project-landing.png
cargo test -p plinth-project
```
