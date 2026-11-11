# Fleetix Hub module — brief, contract and journal

Run `hub-module-2026-11-10`, contract version 1, design plus implementation.
`hub-module.fig` is the design source of truth; `crates/desktop` is the
implementation. Evidence: `/data/nvme0/can/ProjectState/evidence/plinth-hub-module`.

## Brief

**Users.** The operator of Fleetix Hub, who maintains one Plinth personal site
and several `plinth-project` static sites and writes posts for them.

**Product constraints.** Plinth ships a module (`plinth-desktop`, Dioxus
0.8.0-alpha.3), not an application: Fleetix Hub owns the window, sidebar,
topbar and status bar and embeds `BlogWorkspace`. The module uses the hub's
tokens (black / warm charcoal / honey; Space Grotesk, Inter, JetBrains Mono).
Personal sites take posts through the admin API; project sites take a
Markdown file in `posts/` that the new `brick-blog` renders.

**Baseline.** No surface existed. Posts were published with `plinth-cli
publish` per site; project sites could not carry posts at all.

| Task | Start | Outcome | Success check |
| --- | --- | --- | --- |
| T1 pick a site | workspace opens | the selected site, its kind and its existing posts are visible | site row marked current, target chip and posts list name the site |
| T2 draft a post | empty editor | title, slug, description, tags and Markdown body entered with a live preview | slug follows the title until edited; preview renders the body; counter and tag chips update |
| T3 publish to a personal site | valid draft | post accepted by the admin API | receipt with the post URL; post appears in the list; mock API holds the post |
| T4 publish to a project site | valid draft | `posts/<slug>.md` written in the contract format | receipt with the path and the deploy hint; file on disk; `plinth-project build` renders it |
| T5 recover from failure | unreachable site, taken slug, invalid fields | the error names the cause and the draft survives | alert shown, fields intact, retry succeeds once the cause is fixed; draft restored after restart |

Information that must stay available: which site a post goes to before
publishing, the resulting URL or file path, visibility (public or draft), and
the existing slugs of the site.

## Surface and state matrix

- H01 write post, personal site, 1440×900 (write and preview side by side)
- N01 project site, 760×900 (single column, write/preview toggle)
- S01 states: receipt (personal, project), unreachable, slug taken, required
  field, draft visibility, publishing, no sites, no posts, posts failed
- Keyboard: tab order, focus ring, Ctrl+Enter; reduced motion

Predeclared exclusions: light theme and locales other than `en` (the hub has
neither); 390 px (the hub's desktop window does not go that narrow); live
Fleetix Hub integration and any production site (the hub integrates on its
side; evidence uses the `preview` example with a mock API and a temporary
project site); editing or deleting existing posts; Typst and image upload
(remain in `plinth-cli`).

## Journal

Reviewer scores are per rubric category, 0–10. The contract asks for every
category from both final reviewers to reach the private threshold; it has not.

- **Round 0 (design).** H01, S01, N01 authored; fonts faithful.
- **Round 1 (c1).** Delegate implementation reworked to the design. Runtime
  runs exposed dropped keystrokes (controlled inputs echoing through the
  webview), a false "slug taken" on a second publish, and Draft as the default
  visibility; all fixed. Server now returns 409 for a duplicate slug. One
  visual review: revise, 7.0–8.0, no P0/P1. Sol round 1: S1–S3 should, S4 nit,
  all fixed.
- **Round 2 (c2).** Fixed the clipped segmented focus ring, narrow "Write
  another" staying in Preview, implicit form submission on Enter, the receipt
  being the only place the URL or path lived, the unsubmitted draft-visibility
  path, and silent per-site draft swaps. Design frames rebuilt to the final
  copy. Two visual reviews: both revise, 7.0–8.5, no P0/P1. Sol round 2: S2
  still open, S5 new; both fixed.
- **Round 3 (c4, best candidate; c3 was superseded within the round).** Fields
  are read-only while publishing; unsent drafts are held in memory per site;
  the narrow posts disclosure opens on a load failure; an unreachable publish
  no longer claims the post was not stored and reloads the list; the body
  error sits above the textarea; the preview is a focusable region. Not
  visually re-reviewed. Sol round 3: S2 and S5 resolved; S6 blocker (a stale
  in-memory draft could override newer autosaved edits) fixed as suggested
  after the c4 capture and not re-reviewed, so Sol's last verdict stands at
  CHANGES.

- **Round 4 (c5).** Added preview fixtures for the states never captured
  (held publish, rejected key, unwritable `posts/`) and a long-post flow. The
  heading got a rule where fields scroll under it, I/O errors name the target
  file, and the project receipt shows the future URL from `base_url`. Two
  visual reviews: both revise, 7.0–8.5, no P0/P1; four P2 (preview not
  following the caret, clipped long title, invisible personal drafts, small
  nested body field).
- **Round 5 (c7; c6 superseded within the round).** Title wraps; preview
  scroll follows the body; fields are read-only and dimmed while publishing
  and after a receipt, so the receipt no longer disappears on an edit; "Write
  to" / "Written" on project sites; automatic slugs cut at a word; Ctrl+Enter
  works from the whole workspace; selection bar on the selected site; live
  description-limit message; "Draft restored". New flows: keyboard-only site
  selection, restart restore, outage then "Try again", no posts, project slug
  taken and project draft. The capture fonts now really include Space Grotesk
  (fontconfig alias; earlier captures fell back to Inter for display type).
  Two visual reviews: both revise, every category 8.0–8.5, no P0/P1, T1–T5
  pass at runtime.
- **Round 6 (c9, best candidate; c8 superseded by the Sol fixes).** "Discard
  draft" with Undo; the title takes focus when a site is selected; dense
  (seven sites, 43 posts) and 1280/1366 captures. Sol round 4 confirmed S6 and
  raised S7–S12 (draft saved under the wrong site on a fast double switch,
  held drafts not flushed on teardown, draft file-name collisions between
  site ids, failed draft deletion after publish, receipt URL from the
  unnormalised slug, in-flight flag cleared before the outcome was shown);
  all six fixed, not yet re-reviewed. c9 has had no visual review.
- **Round 7 (c10, best candidate).** Sol round 5 on c9: S7–S12 resolved,
  verdict CHANGES with two new findings. S13 (blocker): Ctrl+Enter right after
  a site switch could send the previous site's content to the new site;
  publish and discard now wait until the new site's draft is installed. S14:
  discard reported success when the draft file could not be deleted, and the
  draft came back later; it now deletes first and keeps the draft with an
  error. Full twenty-flow capture, no visual review yet. Sol round 6 on c10:
  S13 and S14 resolved, verdict CHANGES: Undo during a site switch could save
  the discarded draft under the next site (blocker), and Discard ignored the
  in-flight lock (should). Both handlers now refuse while publishing or
  switching. Sol round 7: both resolved, CHANGES: every field and the
  visibility control could do the same during a switch, because `edited()`
  re-enabled saving. `edited()` now ignores edits while the incoming draft is
  being restored. Fixed in the tree after c10 (wide, keyboard and discard flows
  re-run; frames match c10 apart from times and temp paths).
- **Round 8 (c11, best candidate).** Sol round 8 on the three switch guards:
  PASS, no new findings. Full twenty-flow capture frozen as c11. The code
  review loop is closed; the visual contract is not.
- **Round 9 (c12, drafts).** At the user's request: site drafts are listed and
  can be reopened. Server: `GET /api/admin/articles` lists unpublished posts
  with source; `PUT /api/admin/articles/{slug}` replaces a draft in one
  transaction (edit, rename or publish), never a published post; POST still
  refuses any taken slug, so a new post cannot overwrite a draft. PostgreSQL
  test covers list, replace, rename, promote, 404 and 409 (the 409 mapping
  finally has a database test). Module: drafts listed with Open (markdown
  only), "Edit draft" heading, previous editor content one Undo away; project
  drafts are the `published: false` files. New capture flow `drafts`; the
  keyboard flow needed one more Shift+Tab for the new tab stop. Sol round 9:
  CHANGES. Fixed: project replacement now moves the file aside atomically and
  verifies that copy before writing (a post published meanwhile is never
  touched); drafts are replaced by identity, not slug (server draft id under
  `GET/PUT /api/admin/drafts[/{id}]`, project file text), so a stale or
  reused-slug copy is refused; replace keeps author, featured and series; the
  mocks check collisions before changing anything. c12 recaptured on the fix.
  Sol round 10: CHANGES (recovery copy could be clobbered on PID reuse; mock
  reused ids; "published" message for drafts); fixed with a reserved private
  holder directory, ids taken before removal, a draft message. Sol round 11:
  PASS. c12 frozen; it is the best candidate.
  Server tests ran against a throwaway PostgreSQL 18 + pgvector on
  127.0.0.1:55432 (the local role cannot create databases).
- **Round 10 (c13, design caught up).** OpenPencil connected. H01, N01 and
  S01 rebuilt from JSX against `style.css`: bordered site cards with the inset
  selection bar, draft rows with Open, head rule with Discard draft / Undo,
  "Edit draft", bordered segmented controls, dimmed locked visibility,
  "Published" done label, hidden Ctrl+Enter while locked, one-line validation
  banner, project receipt "It will be at", "Try again", muted "Unsent draft
  kept". Drawing the stale-draft state showed a raw 404 with a useless "Try
  again", so the app gained `PublishError::DraftChanged` ("This draft changed
  on <site> after you opened it", nothing written, "Keep as a new post"), and a
  `stale` capture flow (22 flows). c13 frozen with design exports. Tool notes:
  `render` with `replace_id` or `insert_index` put the node at the page root or
  the end; reparent out and back in to order children; `get_jsx` truncates
  long frames at about 12 KB.
- **Round 11 (c14).** c13 reviews: both revise, A 8.0–8.5, B 8.0–9.0, no
  P0/P1; better than c7 on IA, state behaviour and consistency. Sol round 12
  on DraftChanged: CHANGES (stale list on recovery, any 404 read as a changed
  draft, unguarded Keep, overstated wording). Fixed: server answers a gone
  draft with 410 and only 410 maps to DraftChanged; the list reloads; neutral
  wording; Keep checks locks, moves to a free slug from the list
  (`field-notes-2`) and focuses it (reviewers' shared P2). Below 860 px the
  whole workspace scrolls with a sticky result and action bar, so the body no
  longer collapses with many sites or an open result (P2s V13-1/V13-3/F1);
  project receipt path relative to the site root. P3s: Open stays visible but
  disabled while locked, "Editing" chip on the opened draft's row, "Typst"
  chip, heading back to "New post" after a public receipt, input ellipsis,
  centred empty state. Design updated to match (S01 post-row states, stale
  copy, receipt path; N01 status). c14 frozen.
- **Round 12 (c15).** c14 reviews: C revise 7.5–8.5, D revise 7.5–8.5 (D saw
  five captures only), no P0/P1. Sol round 13: CHANGES. Fixed: Editing chip
  matches slug and version, so a newer save shows Open again (also C's P2
  "instruction with no affordance"); focus cannot land under the sticky result
  (`:has()` scroll padding); `free_slug` keeps `-n` within 80 characters (unit
  test); delayed slug focus bound to its editor. From the reviews: "Send
  again" / "Reload posts" instead of two "Try again" (C's P2); empty status
  while sending; "Kept as a new post: <slug>" status; project slug hint
  relative to the root; result panel capped at a third of the window.
  Declined with reasons: project rows stay unlinked (a file in the repo is
  not proof the page is deployed), field errors keep their two positions (the
  body is taller than the window, so its error sits above it), the
  "Visibility" label stays hidden below 860 px (the group keeps its name).
  Contrast measured: muted text #BDAE93 is 5.9:1 to 9.6:1 on every surface,
  control borders #766C5B 3.1:1 to 4.1:1. A bar in long-narrow/04 was a
  WebKitGTK overlay scrollbar that fades after about 2 s; that shot now waits
  4.5 s. Design updated (button labels, behaviour note). c15 frozen.
- **Round 13 (c16).** c15 reviews: E revise 8.5–9.0 (no P0–P2), F revise
  8.0–9.0 (all 85 captures via contact sheets). Sol round 14: CHANGES (Kept
  notice outranked a storage error, outlived the send; `free_slug` sliced
  bytes). Fixed, with a non-ASCII unit case. From the reviews: `public_url`
  for personal sites (links, slug hint, receipts use the public host; the
  former "URLs built from the API address" limit), Dismiss on every error
  panel, sending blocked while "can no longer be replaced" is shown, locked
  fields get a muted focus ring, and two capture steps (an edit clears the
  error; Dismiss by keyboard at 760). Declined: 11 px labels (measured 5.9:1
  or better), body height in error states at 1440 (panel capped at a third).
  Design updated (Dismiss buttons). c16 frozen.
- **Round 14 (c17).** c16 reviews: G revise 8.0–9.0, H revise 8.5–9.0, no
  P0/P1, both better than c15; the shared P2 was the sticky panel covering
  the body at 760. Sol round 15: CHANGES (a locked field's faded ring fell to
  about 2:1; Dismiss dropped keyboard focus). Fixed: a focused locked field
  stays opaque; Dismiss returns focus to Title. The P2: the result panel is
  no longer in the sticky foot; below 860 px CSS `order` puts it under the
  heading, it scrolls with the page, and only the action bar is sticky (an
  error brings itself into view with `scrollIntoView` nearest). "Send again"
  removed: the primary button is the retry (G: two controls for one action;
  H: "Send again" for a file write). Site buttons have a clean accessible
  name. New evidence: AT-SPI accessibility-tree dumps of the running app
  (`a11y/` in the evidence: landmarks, headings, labelled fields, pressed
  states, polite status and assertive alerts, invalid-entry on empty
  publish, keyshortcuts on the primary button) under a private D-Bus session
  with no service activation. Design: N02 (760, publish failed), S01 without
  "Send again", behaviour note. c17 frozen.
- **Round 15 (c18).** c17 reviews: I revise 8.5–9.0, J revise 8.5–9.0, no
  P0–P2; both confirm the c16 P2 fixed. Sol round 16: one should (an error
  scroll could pull a focused invalid field off-screen); validation errors no
  longer scroll. From the reviews: errors and Dismiss bring the editor
  heading into view (the panel follows it at 760), so the site and title
  stay in sight; the post-list failure is a polite status (one assertive
  alert per failed publish); unreachable copy no longer points at a list
  that failed; a selected site card's focus ring is cream, distinct from its
  honey border; locked fields at 0.75 opacity (content 7.6:1, muted 5:1).
  Evidence: the a11y dump prints labelled-by/described-by with text (Title
  described by "! Add a title." after an empty publish); a unit test pins the
  motion rules (no keyframes, 0.15 s transitions, reduced-motion block);
  zoom125-1440 and zoom125-760 captures (whole UI at 125 %). Declined:
  "Write another" repeated in the action bar, a local Ctrl+Enter. c18 frozen
  (24 flows).
- **Round 16 (c19).** c18 reviews: K revise 8.5–9.0, L revise 8.5–9.0, no
  P0/P1. Sol round 17: PASS (nits: the motion test was format-sensitive, and
  dioxus-desktop 0.8.0-alpha.3 drops the scroll alignment options, so
  "nearest" is really "start"; test fixed, caveat recorded in code). K's P2:
  "draft" meant two things; the local copy is now "unsent text" everywhere
  (Clear editor / Editor cleared, Unsent text kept/restored, Autosave
  failed) and "draft" only the site's unpublished post. L's P2s: body and
  preview were cut under a receipt at 1440 (they now shrink to 160 px at
  ≥860 px), and the T4 site renders were stale copies (recipe/render-site.sh
  now builds the project site from each candidate's output with
  plinth-project and screenshots it in headless Chromium; rendered date
  matches the frontmatter). Also: mixed-case accessible names for the
  capitalised section headings; Reload posts inside the unreachable panel.
  Declined: rewriting the absolute path in the read-only error (the path the
  operator needs). c19 frozen.
- **Round 17 (c20).** c19 reviews: M revise 8.5–9.0 (P2: the 760 Dismiss
  capture was mislabelled because one Tab now reached the new Reload posts),
  N revise 8.5–9.0 (P3s only). Sol round 18: CHANGES (an older reload
  response could overwrite a newer one; the 160 px floor cost height in tall
  860–1100 px windows; "Editor cleared" also announced Open replacing
  text). Fixed: reload responses carry a generation; the body grid fills
  spare height (floor kept at ≥860, a result-only floor overflowed at
  1366×768); "Replaced by the opened draft"; a focused invalid field gets a
  cream ring outside its red border; below a 520 px module (125 % zoom in a
  760 window) the primary reads "Publish" and the hint hides; the offline
  narrow flow tabs twice to Dismiss. Checkpoint wording corrected. c20 frozen
  with its own site render and a11y dumps.
- **Round 18 (c21).** c20 reviews: O and P both recommend ready_for_scope
  (no P0–P2; scores 8.5–9.0, still under the 9.5 bar). Sol round 19: one
  should (site switches A→B→A could let an older list win); every list load
  now takes its own number. From the reviews (all P3): heading "Draft
  published" / "Draft saved" after an opened draft is sent; Ctrl+Enter hint
  only beside an enabled primary; project done label always "Written";
  one-line date/slug in post rows; tighter meta below a 520 px module so the
  body shows at 125 % in a 760 window; field-error "!" hidden from assistive
  tech (CSS alt text); progress cursor while sending; the offline fixture has
  a public URL; N02 gains Tags and Description. New evidence: `replace` flow
  ("Replaced by the opened draft", Undo), `tall-1440` (body fills a 1000 px
  window, then yields to the receipt), and a mid-send a11y dump (form busy,
  fields not editable). c21 frozen (26 flows).
- **Round 19 (c22).** Sol round 20: PASS, no findings. c21 reviews: Q
  ready_for_scope 8.5–9.0; R revise 8.5–9.0 with one P2 that c21 itself
  introduced (one-line post rows cut long slugs, which the brief requires to
  stay readable): slugs wrap again. Also: the in-flight primary stays
  focusable (aria-disabled, submit ignores it) so Enter on it keeps focus;
  heading "Draft changed" while the stale panel shows; aria-readonly on the
  locked textareas (WebKitGTK still only drops 'editable' for them);
  description counter described as "N of 160 characters"; thin
  always-visible scrollbars on body and preview so a clipped edge reads as
  scrollable; N02 draws the failed post list. Evidence: a11y/contrast.txt
  with the token ratios (text ≥5.4:1 wherever used, borders ≥3.1:1 on the
  surfaces they sit on). Kept: two Reload posts at 1440 (list and alert).
  c22 frozen.
- **Round 20 (c23).** Sol round 21: PASS, no findings. c22 reviews: S
  ready_for_scope (first 9.5, task clarity; rest 9.0), T ready_for_scope
  (8.0–8.5; P2: a result panel at 1440×900 leaves the body too little
  height). Fixed: compact result panel at ≥860 px (≤25 % of the window, tighter
  spacing) and its Reload posts only below 860 px (the list's own is in
  view); "Sending to <site>…" in the polite status while sending; a taken slug
  gets a short panel like other send errors; network detail in plain words
  ("request to <url> could not connect"); project I/O paths relative to the
  site root; site cards and Open stay focusable mid-send (aria-disabled,
  handlers refuse); the primary drops keyshortcuts when it cannot act;
  contrast.txt says where the 2.5:1 border pair sits (inside the control).
  Kept: the slug input's ellipsis (its hint shows the full path). c23 frozen.
- **Round 21 (c24, final of this run).** Sol round 22: CHANGES (hiding the
  panel's Reload posts at ≥860 px removed the only refresh control when the
  list had loaded; "Replaced by the opened draft" masked the sending status).
  c23 reviews: U ready_for_scope (9.0 ×7, accessibility 8.5), V
  ready_for_scope (8.5–9.0). Fixed: the panel keeps Reload posts at every
  width (the alert carries its remedy, as U and Sol asked); status order is
  autosave error, sending, notices; the status and the in-flight primary use
  the idle verb (Publishing/Writing/Saving); the result panel is at most a
  quarter of the window but never under 12rem; S01/N02 carry the plain
  network wording. c24 frozen; Sol round 23 PASS on the c23→c24 diff (no findings).
- **Round 22 (c25, screen reader).** Orca 50.2 run headless over AT-SPI
  (its debug log records each utterance) through T1–T5 by keyboard. It found
  what the tree dumps could not: the receipt was not read ("Write another"
  was described by its own ancestor, which WebKit resolves to nothing);
  non-field errors were never spoken (Orca ignores nodes added to a live
  region, its spam filter drops an alert inserted within 100 ms of another
  insertion, and this WebKitGTK sends no text-change events for static
  text); Dismiss's move to Title went unreported (focus moved while the
  focused button was being removed); Space on a site card bounced focus back
  to it (Orca refocuses a toggle whose pressed state changes); a taken slug's
  message was not read (an alert inside the field's description). Fixed: the
  receipt text is a sibling the button points at; a non-field error focuses
  its heading (level 3, described by the explanation and detail) so Orca
  reads it whole and a stray key presses nothing (focusing the first panel
  button was tried first: typing a space then pressed it); each send is
  numbered and keys the panel, so a repeated error is read again; Dismiss
  focuses Title before the panel goes and only acts on its own attempt;
  site cards use aria-current; the slug error is plain text. Sol round 24
  (button-focus variant): CHANGES (silent identical retry, stale Dismiss),
  both fixed; Sol round 25 PASS. S01/N02 show the focus ring on the error
  heading. Transcripts in `a11y-orca/`. The working tree was reset to
  `origin/trunk` by another session during this round; the module, server,
  docs and design paths were restored from its stash (the project-site
  `[blog]` brick stayed in that stash, interleaved with the company-site
  work).

- **Round 23 (c26).** c25 reviews: W ready_for_scope (9.0 ×6, consistency
  8.5, accessibility 8.0 on the verified subset), X revise (8.5–9.0,
  accessibility 7.0). Both: the receipt, errors, Dismiss and site switch are
  now spoken; both flagged silent Tabs into Description and an auto-filled
  Slug (F1; X P1, W P2). Logs show a WebKitGTK/Orca event-order race (caret
  move reported before focus; Orca then moves silently), about a third of
  field focus moves in any field; deferring the module's blur bookkeeping did
  not change it (tested, reverted). Fixed from the reviews: the form is named
  with the site ("New post, Pink Raven"), the error heading's ring uses the
  focus token (app and S01/N02), the receipt URL's underline is continuous.
  Sol round 26 PASS. F1 measured and documented in `a11y-orca/README.md`.

## Checkpoint

- **Stop reason:** the contract (9.5 in every category from both reviewers)
  is not met. With a real screen-reader run (c25/c26) the remaining cap is
  accessibility: about a third of Tab moves into a text field are not
  announced, a WebKitGTK/Orca event-order race the module cannot reorder.
  Other categories sit at 8.5–9.0 with only P3s, which earlier rounds showed
  reverse between reviewers.
- **Best candidate:** c26, evidence and `manifest.sha256` under
  `/data/nvme0/can/ProjectState/evidence/plinth-hub-module/c26`. Raw reviewer
  reports are in the session transcript, not in that directory.
- **Capture recipe:** `recipe/run-all.sh APP_OUT_DIR` in the evidence
  directory, run inside the infernix-hub devshell: 26 flows on private
  Xvfb displays at 96 dpi with the hub's fonts through fontconfig, driven by
  xdotool. Headless Wayland (sway) gave WebKitGTK a negative device scale.
  Screen reader: `recipe/a11y/run-orca.sh FIXTURE OUT.log STEPS` under
  `dbus-run-session --config-file=recipe/a11y/session.conf` runs Orca 50.2
  with a debug log of every utterance; stop Orca before the bus, or its
  buffered log is cut short.
- **Design export:** from the running app (MCP `export_image`); the
  file-backed `openpencil export` 0.15.1 mis-measures semibold Inter.
- **Subagents:** implementer — T3 `delegate_task`, opencode
  `opencode-go/deepseek-v4.1-flash` variant max for the first implementation;
  the orchestrator did the design and the refinement rounds. Reviewer — Claude
  Code `visual` subagent, fresh context per review; Sol
  (`openai/gpt-6.1-sol`, xhigh) for code.
- **Unresolved:**
  - posts published here carry no embedding (absent from semantic search
    until republished with `plinth-cli`);
  - link activation (`target=_blank`) in the hub webview is untested;
  - the rejected-key message cannot name the key file (`Site` keeps only the
    key);
  - the screen-reader run is Orca's own transcript, not a person listening
    (no speech synthesis; one environment quirk: the first Tab into a
    prefilled input is sometimes silent); no cancel while publishing;
  - `plinth-project` renders the posts the module writes only with the
    project-site `[blog]` brick, which lands in its own PR right after this
    one;
  - `nix flake check` not run (the Plinth devshell currently fails to build
    `plinth-project`, outside this work).
- **Decision (2026-11-11):** the user accepted c26 as ready for scope. The
  silent-Tab race stays an upstream WebKitGTK/Orca issue, documented in
  `c26/a11y-orca/README.md`.
- **Next action:** none for the design run. The work is committed and lands
  through its own PR; the `[blog]` brick and the company-site work follow as
  separate PRs. Council review before merge added the declarative-draft guard
  (the drafts API lists and replaces only API posts), `create_new` and fsync
  for project post writes, and a flake check that builds, lints and tests
  `plinth-desktop`, which the root workspace gates never compiled.
