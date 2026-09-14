# Local history integration, 2026-09-14

Destination: local `trunk`. No Git branches were pushed, deleted, or rebased.
The integration was prepared on `integration/sync-20260914` before promotion.
Original worktree branches and recovery refs are retained.

## Recovery

Before fetching, all refs (including tree checkpoints) and the detached Simit
commit were preserved in the verified bundle:
`/data/scratch/tmp/opencode/plinth-before-sync-20260914.bundle`.
`recovery/sync-20260914-simit` names detached commit
`f5c5998d90af3238f1199b3d25ce4cf457168cd8`.

The fetched GitHub trunk was `bbd66f86d167edb69a512712b3fd242051f80256`.
Cached `origin/trunk` (`87944d5`) was stale; its GitHub migration work was
integrated separately. Local `trunk` (`79a5be7`) and fetched trunk have equal
source trees except for two rewritten lockfile revision IDs.

## Commit histories

All local branches, remote-tracking branches, peer branch snapshots, salvage
commits, the tag, stash, and detached Simit commit are ancestors of the candidate.
Identical refs and ancestors need no additional merge.

- Project grid: retained the brick, configuration, rendering, CSS and tests.
- Dioxus branches: retained 0.7.10 and the pinned matching nixpkgs revision;
  retained the host-package PostgreSQL and generated metadata fixes.
- WIP: retained Harbor rename/pin, compiler-cache wrapper path, generated
  Graphify removal, blog-writer simplification and portfolio rename. Kept
  project-grid CSS and the newer Dioxus version through conflicts.
- GitHub migration: retained canonical project URLs and refreshed public
  input locks; did not revert to legacy CodeFloe infrastructure.
- Attic branch: retained the publication app, aggregate CI, loader cleanup
  and corrected cache-control expectations.
- Detached Simit migration: reconciled against the later aggregate CI and
  Harbor-owned Attic input publication. The obsolete per-crate duplicate
  workflows and malformed inline Attic configuration were not restored.
- Legacy rescue: retained removal of committed browser dependencies and
  generated HTML, while preserving newer toolchain integration.
- Salvaged rescue and WIP: source-equivalent rewrites, differing only in
  obsolete lockfile revision IDs. Kept the newer resolved lock graph.
- Backup: equivalent old blog-writer guidance; preserved the canonical skill
  symlink rather than replacing it with a historical directory.
- Stash: preserved the stash and its parent history, including untracked
  Graphify recovery data. Generated cache files remain outside the source tree.
- Pages: joined unrelated deployment history, without restoring its generated
  HTML, JavaScript and font files into the source checkout. Source-owned site
  and documentation builders remain authoritative.

## Raw checkpoint reconciliation

These 13 refs contain ten distinct **full tree snapshots**, not commits.
They are compared below against nearby commit trees, ignoring `graphify-out/`
when measuring source differences. Comparison commits are evidence anchors,
not claims about the snapshots' original parents. No parentage or original
author metadata was fabricated.

| Tree | Comparison commit | Source paths differing | Resolution |
| --- | --- | ---: | --- |
| `537149b9127f578740703d5891e003c899172133` | `a543d6b` | 2 | Historical cutover planning text; later reconciled planning text retained. |
| `6221412cfa7373336b0e0230c13de68540bca16d` | `d827e7c` | 2 | Pre-helper flake and earlier planning state; retained shipped Dioxus helper and OpenVINO cross-build exclusion. |
| `2c351f83e9a25dc76d381f8bd21fef24e3b87a1f` | `55bc7db` | 0 | Source already in history; only generated Graphify state differs. |
| `927ca93be4265ebe1158d6b021026fd0f248eb93` | `55bc7db` | 5 | Superseded local sccache version/wrapper experiment; retained the explicit Harbor-owned cache contract instead. |
| `a697ce88b7a211ed5e9228b46707b06c95fae82a` | `2517782` | 4 | PostgreSQL 17, database user URLs and backup ignore rule already carried forward; retained newer Dioxus streaming test. |
| `495177cb761dc94c560ad2d6f4c8e0c667b183cf` | `ab10ade` | 1 | Module changes already carried forward, with subsequent peer-authentication and restart hardening retained. |
| `deede9138a667161131f40b3fa6f3e41f86db8a0` | `ab10ade` | 1 | Same module source state as `495177c`; generated Graphify state differs. |
| `1aeac36707f1515c117676cb8e8fc24563fec0f1` | `d827e7c` | 2 | Pre-helper flake/planning snapshot; newer implementation retained. |
| `d5de609e98105454b78ec8d6a1d3e97b800ecfa5` | `d827e7c` / `c55fd54` | 0 | Exact complete commit tree match, already in history. |
| `afe4e946021d29897055c1f0882282fbf36e4c20` | `a543d6b` | 2 | Earlier cutover plans; retained subsequent reviewed planning state. |

No additional source restoration was needed after reconciling these deltas.
All original raw-tree refs and their exact objects remain available, including
generated cache data. Reconciliation is not a claim that obsolete snapshots
were copied over the final checkout.

## Verification

Run `bash scripts/check-history-integration.sh` to verify the recorded commit
tips are ancestors of HEAD and the original checkpoint tree objects still exist.
This checks history preservation, not application correctness. Application
checks must be run separately through the repository's Nix check outputs.
