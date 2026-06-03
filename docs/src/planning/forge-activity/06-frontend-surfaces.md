# Phase 06 — Frontend: /activity pages + home feature strip

> **Recommended Codex model: GPT 5.5 medium**
>
> This phase is moderate: it is a near-mechanical mirror of the existing portfolio
> Leptos surfaces (list page, detail page, `#[server]` functions, route registration,
> home-page section), but it has three real traps that a too-small model will fall
> into and produce code that does not compile. (1) The route table is built from **two
> whole-tuple `app_routes()` functions** gated by `#[cfg(all(...))]` / `#[cfg(not(all(...)))]`,
> not per-`<Route>` `#[cfg]` — adding `brick-activity` to the `all(...)` list silently
> drops every brick route in the *default* build unless done correctly. (2) Leptos
> `view!` `match` arms must all share one type, which is why every branch is wrapped in
> `EitherOf3::{A,B,C}` — a smaller model tends to return bare `view!` from each arm and
> hit a type-mismatch wall. (3) The `ActivityListItem` wire type carries a **computed
> `score: f64`** plus enum fields (`Forge`, `ActivityKind`, `ActivityState`) that render
> differently from portfolio's plain strings. A medium model can hold all three
> constraints at once while copying the proven pattern; a low tier would either break the
> cfg matrix or fabricate fields that do not exist on the shared types.

## Working tree

- `cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).
- This is **Wave 2**. It depends only on **Phase 03** (the public server API + the
  `ActivityItem` / `ActivityListItem` shared types). You consume the contract; you do
  **not** implement the server side here.
- **No serialization conflict with siblings.** This phase touches only `crates/client/**`
  (and, optionally, two leaf files in `crates/shared/src/config.rs` if you choose to add a
  configurable page title). Phase 04 (server `cache.rs`/`refresh.rs`), Phase 05 (`crates/cli`),
  and Phase 07 (server `search`/`feeds`/`main.rs`) touch disjoint files. You will not
  rebase against them.
- **Before starting, confirm Phase 03 landed:** the shared types `plinth_shared::ActivityItem`,
  `plinth_shared::ActivityListItem`, and the enums `Forge`, `ActivityKind`, `ActivityState`
  must exist and be exported under `#[cfg(feature = "brick-activity")]`, and the public
  endpoints `GET /api/activity` and `GET /api/activity/{id}` must be wired. If they are
  not, the `#[server]` functions you add here will fail to compile (the SSR body calls into
  the activity cache actor) or return errors at runtime. See `./03-server-brick-core.md`
  for sequencing only — do not implement server code from this file.

## Goal

This phase succeeds when, with the `brick-activity` feature enabled, the Leptos client
compiles to **both** native SSR and the `wasm32-unknown-unknown` hydrate target; the
routes `/activity` and `/activity/:id` resolve to real page components; the `/activity`
page renders a ranked grid of contributions (each showing impact, forge, state, reference
date, and an outbound link to the upstream PR/issue URL); the `/activity/:id` page renders
a single contribution detail; and the home page shows a top-N "Recent Activity" feature
strip ordered by score — all served **without any authentication** (these are public
reads against `GET /api/activity`).

## Why this matters now

The activity brick's entire user-facing value is the curated, impact-ranked view of the
owner's external contributions. Phases 01–05 build the data plane (types, forge fetching,
persistence, ranking SQL, CLI ingest) but produce **zero** visible surface. This phase is
the first one a visitor can actually see: the dedicated `/activity` page (surface 4a), the
home-page strip (surface 4b). Deferring it would mean the brick exists in the database and
API but is invisible, blocking any review of ranking quality or copy. It also unblocks
Phase 08's e2e tests, which navigate to `/activity` and assert the rendered list. The feed
(`/feeds/activity.xml`) and search union are surfaces 4c/4d and belong to **Phase 07** —
not here.

## Out of scope

- **The server API itself.** Do not write `crates/server/src/bricks/activity/api.rs`,
  `cache.rs`, `admin.rs`, the migration, or ranking SQL. Those are Phase 03 (already
  landed) / Phase 04. The `#[server]` functions you add are the *client-side* bridge whose
  SSR body asks the existing `activity_cache` actor — but **Phase 03 does not touch
  `crates/client`**, so you write those real SSR bodies here (step 3), mirroring the
  portfolio `#[server]` bodies exactly. There is no `todo!()` deferral in this phase.
- **RSS/Atom feed** (`/feeds/activity.xml`) and **search union** — Phase 07. Do not touch
  `crates/server/src/api/feeds.rs`, `crates/server/src/api/search.rs`,
  `crates/server/src/actors/vector_search.rs`, or `crates/server/src/main.rs`.
- **The lazy refresh actor / TTL / single-flight** — Phase 04. Do not touch
  `crates/server/src/bricks/activity/cache.rs` or `refresh.rs`.
- **CLI** — Phase 05. Do not touch `crates/cli/**`.
- **Defining the shared types.** `ActivityItem`, `ActivityListItem`, `Forge`,
  `ActivityKind`, `ActivityState`, `RankingStrategy` are owned by Phase 01/03 in
  `crates/shared/src/`. You only *consume* them. If a field you need is missing, that is a
  Phase 01 gap — note it; do not add fields to the shared crate from this phase beyond the
  optional `ActivityPageConfig` in step 8 (which is page-display config, not a data type).
- **Server-side `Cargo.toml` / workspace `bin-features`.** This phase's only Cargo edits
  are the client and (transitively) shared feature flags so the WASM build picks up
  `brick-activity`. The server brick feature wiring is Phase 03's job; the workspace
  `bin-features` list is shared across phases — only add `brick-activity` there if it is not
  already present (it is added by whichever of 03/05/06 lands first; check, don't duplicate).

## Plan

> All snippets below are grounded in the live portfolio brick. The activity surfaces are a
> 1:1 structural copy with field substitutions. Where a snippet shows the portfolio
> original for reference, the activity version follows.

### 1. Confirm the shared wire-type contract (read-only)

The frontend renders these fields. They come from Phase 01/03 in `crates/shared/src/`
(mirroring `crates/shared/src/portfolio_item.rs`). Inline contract you must rely on:

```rust
// plinth_shared, gated by `#[cfg(feature = "brick-activity")]`
pub enum Forge { GitHub, Codeberg }
pub enum ActivityKind { PullRequest, Issue }
pub enum ActivityState { Open, Closed, Merged }

pub struct ActivityListItem {   // ranked list / home strip / feed
    pub id: i64,
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,            // upstream PR/issue URL — link target
    pub title: String,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub impact: i16,            // 1..=10
    pub labels: Vec<String>,
    pub featured: bool,
    pub score: f64,            // computed at READ time by the ranking SQL
}
impl ActivityListItem {
    // The reference date is NEVER a stored column — always derive it via this helper.
    pub fn reference_date(&self) -> DateTime<Utc> {
        self.merged_at.or(self.closed_at).unwrap_or(self.created_at)
    }
}

pub struct ActivityItem {       // full row (admin + public detail); does NOT carry the embedding
    pub id: i64,
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,
    pub title: String,
    pub body: Option<String>,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub impact: i16,
    pub additions: Option<i32>,
    pub deletions: Option<i32>,
    pub comments_count: Option<i32>,
    pub labels: Vec<String>,
    pub repo_stars: Option<i32>,
    pub fetched_at: DateTime<Utc>,
    pub featured: bool,
    pub published: bool,
    pub content_hash: Option<String>,
}
```

`ActivityItem` carries the raw PR/issue `body: Option<String>` (the brief stores `body
TEXT`). Render it as plain text inside a `<p>`/`<pre>`. There is **no** `html_content`
field on `ActivityItem` — do not assume one exists; render `body` as text.

The reference date is **never** a stored field on either type — always derive it via the
Phase 01 helper `ActivityListItem::reference_date()` (defined in `crates/shared/src/`).
Do not re-implement the coalesce inline in the client. Only `forge_label` / `state_label`
are local display closures you define per page:

```rust
fn forge_label(f: &plinth_shared::Forge) -> &'static str {
    match f { plinth_shared::Forge::GitHub => "GitHub", plinth_shared::Forge::Codeberg => "Codeberg" }
}
fn state_label(s: &plinth_shared::ActivityState) -> &'static str {
    match s {
        plinth_shared::ActivityState::Merged => "Merged",
        plinth_shared::ActivityState::Closed => "Closed",
        plinth_shared::ActivityState::Open => "Open",
    }
}
// Reference date comes from the shared helper, NOT a local copy:
//   let ref_date = item.reference_date();   // merged_at.or(closed_at).unwrap_or(created_at)
```

### 2. Add the client + shared `brick-activity` feature flags

`crates/client/Cargo.toml` — mirror the portfolio line (current `[features]` block):

```toml
[features]
default = ["csr", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
brick-blog = ["plinth-shared/brick-blog"]
brick-portfolio = ["plinth-shared/brick-portfolio"]
brick-todo = ["plinth-shared/brick-todo"]
brick-activity = ["plinth-shared/brick-activity"]   # <-- add
csr = ["leptos/csr"]
hydrate = ["leptos/hydrate"]
ssr = ["dep:serde_json"]
```

`crates/shared/Cargo.toml` — Phase 01 should already have added `brick-activity = []` and
appended it to `default`. If absent (you are running before 01's Cargo edit landed),
add the leaf marker:

```toml
brick-activity = []
```

Workspace `Cargo.toml` cargo-leptos lists (root, lines ~132 and ~138) — these compile the
feature into the server bin **and** the WASM lib. Add `brick-activity` **only if not
already present** (Phase 03/05 may have added it):

```toml
bin-features = ["ssr", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
lib-features = ["hydrate", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
```

### 3. Add the two `#[server]` functions in `crates/client/src/api.rs`

Mirror the portfolio block (`api.rs:67-82`) exactly. **Write the real SSR bodies here** —
Phase 03 does **not** touch `crates/client`, so these bodies are this phase's
responsibility, not a `todo!` deferral. Both functions ask the activity cache actor (which
Phase 03 placed on `AppState` as `activity_cache: ActorRef<ActivityCache>`) using the
canonical kameo messages from the contract:

- list  → `GetRankedActivity { limit: Option<i64>, featured_only: bool }` → `Vec<ActivityListItem>`
- detail → `GetActivityItem(i64)` → `Option<ActivityItem>`

The two `#[server]` macro names are `GetActivityList` (list) and `GetActivityItemById`
(detail) — deliberately distinct from the actor message `GetRankedActivity`. Append after
the portfolio section:

```rust
// ── Activity server functions ───────────────────────────────────────────────

#[cfg(feature = "brick-activity")]
#[server(GetActivityList, "/api")]
pub async fn get_activity_list() -> Result<Vec<plinth_shared::ActivityListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::AppState;
        use plinth_server::bricks::activity::cache::GetRankedActivity;
        let state = expect_context::<AppState>();
        let items = state
            .activity_cache
            .ask(GetRankedActivity { limit: Some(50), featured_only: false })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(items)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-activity")]
#[server(GetActivityItemById, "/api")]
pub async fn get_activity_item_by_id(
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::AppState;
        use plinth_server::bricks::activity::cache::GetActivityItem;
        let state = expect_context::<AppState>();
        let item = state
            .activity_cache
            .ask(GetActivityItem(id))
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(item)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        unreachable!("server fn body only runs under ssr")
    }
}
```

Notes:
- The public endpoints are `GET /api/activity` (ranked list) and `GET /api/activity/{id}`.
  The `#[server(..., "/api")]` prefix matches the existing convention. The SSR bodies above
  reach the same data through the `activity_cache` actor rather than an HTTP round-trip,
  exactly as the portfolio server fns do (mirror the precise `expect_context` /
  module path the portfolio block resolves to in the live tree).
- `limit: Some(50)` caps the ranked list for the public page; `featured_only: false`
  returns the full ranked set. The home strip (step 9) calls the same fn and `.take(4)`s
  the prefix — it does not need a separate query.
- The kameo `ask(...)` returns a `SendError`; map it to `ServerFnError::new(e.to_string())`
  so the `Result<_, ServerFnError>` signature is satisfied and the page surfaces an error
  instead of panicking.
- Activity is keyed by **numeric `id`** (`BIGSERIAL`), not a slug — this is the one place
  the activity surface diverges from portfolio (which keys by slug). The detail route is
  `/activity/:id` and the param is parsed as `i64`; the `#[server]` detail fn takes
  `id: i64`.

### 4. Create `crates/client/src/pages/activity.rs` (ranked list/grid)

Copy `crates/client/src/pages/portfolio.rs` and substitute fields. Full file:

```rust
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

fn forge_label(f: &plinth_shared::Forge) -> &'static str {
    match f {
        plinth_shared::Forge::GitHub => "GitHub",
        plinth_shared::Forge::Codeberg => "Codeberg",
    }
}
fn state_label(s: &plinth_shared::ActivityState) -> &'static str {
    match s {
        plinth_shared::ActivityState::Merged => "Merged",
        plinth_shared::ActivityState::Closed => "Closed",
        plinth_shared::ActivityState::Open => "Open",
    }
}

#[component]
pub fn ActivityPage() -> impl IntoView {
    let config = use_site_config();
    let page_title = "Activity".to_string();
    let title_text = format!("{} - {}", page_title, config.name);

    let canonical_url = if config.base_url.is_empty() {
        "/activity".to_string()
    } else {
        format!("{}/activity", config.base_url)
    };

    let items = Resource::new(|| (), |_| async move { api::get_activity_list().await });

    view! {
        <Title text={title_text}/>
        <Meta name="description" content="Curated external contributions across GitHub and Codeberg, ranked by impact and recency."/>
        <Link rel="canonical" href={canonical_url}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16">
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        {page_title}
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-amber-400">
                        "Contributions I have landed on other people\u{2019}s projects."
                    </p>
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-amber-400">"Loading activity..."</p>
                    </div>
                }>
                    {move || {
                        items.get().map(|result| {
                            match result {
                                Ok(items) => {
                                    if items.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-amber-400">
                                                    "No activity yet. Check back soon!"
                                                </p>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                                                {items.into_iter().map(|item| {
                                                    let id = item.id;
                                                    let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                                                    let ref_date = item.reference_date();
                                                    view! {
                                                        <a
                                                            href={format!("/activity/{}", id)}
                                                            class="card card-dark block hover:scale-105 transition-transform"
                                                        >
                                                            <div class="p-6">
                                                                <div class="flex items-center justify-between mb-2 text-sm text-gray-500 dark:text-amber-400">
                                                                    <span>{forge_label(&item.forge)}</span>
                                                                    <span class="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200">
                                                                        {state_label(&item.state)}
                                                                    </span>
                                                                </div>
                                                                <h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-amber-100">
                                                                    {item.title.clone()}
                                                                </h2>
                                                                <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                                    {repo} " #" {item.number}
                                                                </p>
                                                                <div class="flex items-center justify-between text-sm">
                                                                    <span class="text-gray-500 dark:text-amber-400">
                                                                        {ref_date.format("%b %Y").to_string()}
                                                                    </span>
                                                                    <span class="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200 rounded-full">
                                                                        "Impact " {item.impact}
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        })
                                    }
                                },
                                Err(_) => EitherOf3::C(view! { <ErrorMessage/> }),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
```

Notes:
- The list arrives **already ranked** from the server (`ORDER BY score DESC, reference_date
  DESC`). Do **not** re-sort on the client — render in received order. The `score` field is
  present on `ActivityListItem` but you do not need to display it (impact is the visible
  proxy); the home strip relies on the same server ordering.
- The card links to the internal detail page (`/activity/{id}`), matching portfolio's
  `/projects/{slug}` linking. The *upstream* URL is the primary link on the **detail** page
  (step 5); on the grid, the card itself is the internal nav target.

### 5. Create `crates/client/src/pages/activity_detail.rs` (detail page)

Copy `crates/client/src/pages/portfolio_detail.rs` and adapt the param to `i64` and the
fields to activity. Full file:

```rust
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

fn forge_label(f: &plinth_shared::Forge) -> &'static str {
    match f {
        plinth_shared::Forge::GitHub => "GitHub",
        plinth_shared::Forge::Codeberg => "Codeberg",
    }
}
fn state_label(s: &plinth_shared::ActivityState) -> &'static str {
    match s {
        plinth_shared::ActivityState::Merged => "Merged",
        plinth_shared::ActivityState::Closed => "Closed",
        plinth_shared::ActivityState::Open => "Open",
    }
}

#[component]
pub fn ActivityDetailPage() -> impl IntoView {
    let params = use_params_map();
    // Param is the numeric id; parse to i64 (0 on parse failure -> server returns None).
    let id = move || {
        params
            .with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()))
            .unwrap_or(0)
    };

    let item = Resource::new(id, |id| async move { api::get_activity_item_by_id(id).await });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                    <p class="text-gray-600 dark:text-amber-400">"Loading contribution..."</p>
                </div>
            </div>
        }>
            {move || {
                item.get().map(|result| {
                    match result {
                        Ok(Some(item)) => {
                            let config = use_site_config();
                            let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                            let ref_date = item.merged_at
                                .or(item.closed_at)
                                .unwrap_or(item.created_at);
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", item.title, config.name)}/>
                                <Meta name="description" content={item.title.clone()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        <a href="/activity" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} Back to Activity"
                                        </a>

                                        <header class="mb-8">
                                            <div class="flex flex-wrap items-center gap-3 mb-4 text-sm text-gray-600 dark:text-amber-400">
                                                <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full">
                                                    {forge_label(&item.forge)}
                                                </span>
                                                <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full">
                                                    {state_label(&item.state)}
                                                </span>
                                                <span class="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200 rounded-full">
                                                    "Impact " {item.impact}
                                                </span>
                                            </div>

                                            <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100 leading-tight">
                                                {item.title.clone()}
                                            </h1>

                                            <p class="text-lg text-gray-600 dark:text-amber-400 mb-2">
                                                {repo} " #" {item.number}
                                            </p>
                                            <p class="text-gray-500 dark:text-amber-400 mb-6">
                                                {ref_date.format("%B %e, %Y").to_string()}
                                            </p>

                                            <a
                                                href={item.url.clone()}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="btn-primary inline-flex items-center gap-2"
                                            >
                                                "View on " {forge_label(&item.forge)} " \u{2197}"
                                            </a>
                                        </header>

                                        // Body — rendered as plain text (the brief stores raw `body TEXT`).
                                        {item.body.as_ref().map(|body| view! {
                                            <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-black rounded-lg shadow-lg p-8 mb-12">
                                                <p class="whitespace-pre-wrap">{body.clone()}</p>
                                            </div>
                                        })}

                                        <footer class="mt-8 pt-8 border-t border-gray-200 dark:border-amber-900/50">
                                            <a href="/activity" class="btn-secondary">
                                                "\u{2190} All Activity"
                                            </a>
                                        </footer>
                                    </article>
                                </div>
                            })
                        },
                        Ok(None) => EitherOf3::B(view! {
                            <Title text="Contribution Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Contribution Not Found"
                                    </h1>
                                    <a href="/activity" class="btn-primary">"View All Activity"</a>
                                </div>
                            </div>
                        }),
                        Err(_) => EitherOf3::C(view! { <ErrorMessage/> }),
                    }
                })
            }}
        </Suspense>
    }
}
```

> If Phase 01 produced `html_content: Option<String>` on `ActivityItem` instead of raw
> `body`, replace the body block with portfolio's idiom:
> `{item.html_content.as_ref().map(|html| view! { <div inner_html={html.clone()}></div> })}`.

### 6. Register the page modules + re-exports in `crates/client/src/pages/mod.rs`

Mirror the portfolio entries (`mod.rs:17-20` and `mod.rs:48-51`). Add the module
declarations:

```rust
#[cfg(feature = "brick-activity")]
mod activity;
#[cfg(feature = "brick-activity")]
mod activity_detail;
```

…and the re-exports:

```rust
// Activity pages
#[cfg(feature = "brick-activity")]
pub use activity::ActivityPage;
#[cfg(feature = "brick-activity")]
pub use activity_detail::ActivityDetailPage;
```

`crates/client/src/app.rs` already does `use crate::pages::*;` (line 9), so the page
components become visible to the route table automatically.

### 7. Register the routes in `crates/client/src/app.rs`

This is the load-bearing trap. `app_routes()` exists in **two** `#[cfg]` variants
(`app.rs:78-101` all-bricks; `app.rs:106-121` minimal fallback). The `<Routes>` tuple must
be statically known, so you cannot put `#[cfg]` on an individual `<Route>`.

**Step 7a — add `brick-activity` to BOTH cfg gate lists** so the all-bricks variant
remains the one selected by the default feature set:

```rust
#[cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
))]
fn app_routes() -> impl IntoView { /* ... */ }

#[cfg(not(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
)))]
fn app_routes() -> impl IntoView { /* minimal fallback */ }
```

**Step 7b — add the two routes inside the all-bricks `app_routes()` body**, next to the
portfolio routes (`app.rs:94-95`):

```rust
<Route path=path!("/activity") view=ActivityPage/>
<Route path=path!("/activity/:id") view=ActivityDetailPage/>
```

`use leptos_router::path;` is already imported (`app.rs:4`); the `path!("/activity/:id")`
macro names the `id` param read by `use_params_map` in step 5.

> Rationale for editing the gate list: the default feature set enables all four bricks, so
> the `all(...)` variant is the compiled one and now carries the activity routes. The
> minimal fallback variant deliberately lists only core pages (`/`, `/about`, `/support`);
> activity routes do not belong there because a build without `brick-activity` has no
> `ActivityPage` symbol. By adding `brick-activity` to **both** cfg expressions, a build
> that enables every brick *except* activity correctly falls into the fallback variant and
> compiles (no dangling `ActivityPage` reference).

### 8. (Optional) Configurable page title via `SiteConfig`

The page above hardcodes `"Activity"`. To match the portfolio convention
(`config.pages.portfolio.title`), you may add an `ActivityPageConfig` to the **client-safe**
config in `crates/shared/src/config.rs`, mirroring `PortfolioPageConfig` (`config.rs:107-130`)
and wiring it into `PagesConfig` (`config.rs:208-221`):

```rust
/// Activity page configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityPageConfig {
    #[serde(default = "default_activity_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}
impl Default for ActivityPageConfig {
    fn default() -> Self {
        Self { title: default_activity_title(), subtitle: String::new(), description: String::new() }
    }
}
fn default_activity_title() -> String { "Activity".to_string() }
```

Then add `#[serde(default)] pub activity: ActivityPageConfig,` to `PagesConfig` and (if the
server projects it) the matching `PagesTomlConfig` field in
`crates/shared/src/toml_config.rs`. **This is optional polish** — it is the only edit
outside `crates/client/**` this phase may make, and it is a pure leaf addition with no
serialization conflict. If you do it, swap the hardcoded `page_title` in `activity.rs` for
`config.pages.activity.title.clone()`. If you skip it, leave the hardcoded title.

### 9. Add the home-page "Recent Activity" strip in `crates/client/src/pages/home.rs`

Mirror the `portfolio_section()` / `#[cfg(not(...))]` pair (`home.rs:154-229`).
`EitherOf3` and `api` are already imported at the top of `home.rs`.

**Step 9a — call it in the view body**, after the existing sections (`home.rs:78`):

```rust
                {blog_section()}
                {portfolio_section()}
                {activity_section()}   // <-- add this line
```

**Step 9b — define the gated function** (append near the other sections):

```rust
/// Recent Activity strip — top-N by score; only compiled when brick-activity is enabled.
#[cfg(feature = "brick-activity")]
fn activity_section() -> impl IntoView {
    let items = Resource::new(|| (), |_| async move { api::get_activity_list().await });

    view! {
        <section class="mb-16">
            <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400 mb-6">
                "Recent Activity"
            </h2>

            <Suspense fallback=move || view! {
                <p class="text-gray-500 dark:text-amber-400">"Loading..."</p>
            }>
                {move || {
                    items.get().map(|result| {
                        match result {
                            Ok(items) => {
                                if items.is_empty() {
                                    EitherOf3::A(view! {
                                        <p class="text-gray-500 dark:text-amber-400">"No activity yet."</p>
                                    })
                                } else {
                                    // Server already returns ranked (score DESC); take top-N.
                                    let items: Vec<_> = items.into_iter().take(4).collect();
                                    EitherOf3::B(view! {
                                        <div class="space-y-4">
                                            {items.into_iter().map(|item| {
                                                let id = item.id;
                                                let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                                                view! {
                                                    <a
                                                        href={format!("/activity/{}", id)}
                                                        class="flex items-baseline justify-between gap-4 group py-2"
                                                    >
                                                        <span class="text-gray-900 dark:text-amber-100 group-hover:text-blue-600 dark:group-hover:text-amber-200 transition-colors">
                                                            {item.title}
                                                        </span>
                                                        <span class="text-sm text-gray-400 dark:text-amber-600 shrink-0">
                                                            {repo}
                                                        </span>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    })
                                }
                            },
                            Err(_) => EitherOf3::C(view! {
                                <p class="text-gray-500 dark:text-amber-400">"Could not load activity."</p>
                            }),
                        }
                    })
                }}
            </Suspense>

            <a href="/activity" class="inline-block mt-4 text-sm text-blue-600 dark:text-amber-300 hover:underline">
                "All activity \u{2192}"
            </a>
        </section>
    }
}

#[cfg(not(feature = "brick-activity"))]
fn activity_section() -> impl IntoView { () }
```

**Top-N choice:** `.take(4)` matches portfolio's `.take(3)` idiom (small home strip). The
server returns the list already sorted `score DESC`, so taking the prefix yields the
top-N by score — **do not re-sort on the client**.

### 10. Build verification

```bash
# Native SSR build with the new feature (server-side server-fn bodies must compile):
cargo check -p plinth-client --no-default-features --features "ssr,brick-activity"

# WASM hydrate target — the real client surface (this is the load-bearing check):
cargo check -p plinth-client --target wasm32-unknown-unknown \
    --no-default-features --features "hydrate,brick-activity"

# Whole-workspace clippy as the flake's `plinth-clippy` check runs it (warnings = errors):
cargo clippy --workspace --all-targets -- --deny warnings

# Full app build the way `cargo leptos build` (and the flake) does, picking up
# bin-features/lib-features that now include brick-activity:
cargo leptos build
```

The `wasm32-unknown-unknown` target must already be installed (the flake's toolchain
includes it). If missing locally: `rustup target add wasm32-unknown-unknown`.

## Acceptance criteria

- [ ] `cargo check -p plinth-client --target wasm32-unknown-unknown --no-default-features --features "hydrate,brick-activity"` succeeds with **zero errors** (the WASM client surface compiles with the brick on).
- [ ] `cargo check -p plinth-client --no-default-features --features "ssr,brick-activity"` succeeds (SSR server-fn bodies compile).
- [ ] `cargo clippy --workspace --all-targets -- --deny warnings` produces **0 warnings** (the flake `plinth-clippy` gate).
- [ ] `cargo leptos build` succeeds, compiling `brick-activity` into both `bin-features` and `lib-features` (no `ActivityPage`/`ActivityDetailPage`/`activity_section` "cannot find" errors).
- [ ] `crates/client/src/pages/activity.rs` exists and exports `ActivityPage`; `crates/client/src/pages/activity_detail.rs` exists and exports `ActivityDetailPage`; both are `#[cfg(feature = "brick-activity")]`-gated in `pages/mod.rs`.
- [ ] `crates/client/src/app.rs` registers `<Route path=path!("/activity") view=ActivityPage/>` and `<Route path=path!("/activity/:id") view=ActivityDetailPage/>` inside the all-bricks `app_routes()`, and `brick-activity` appears in **both** the `#[cfg(all(...))]` and `#[cfg(not(all(...)))]` gate lists.
- [ ] `crates/client/src/api.rs` defines `#[server(GetActivityList, "/api")]` returning `Result<Vec<plinth_shared::ActivityListItem>, ServerFnError>` and `#[server(GetActivityItemById, "/api")]` taking `id: i64` and returning `Result<Option<plinth_shared::ActivityItem>, ServerFnError>`, both `#[cfg(feature = "brick-activity")]`-gated, with **real SSR bodies** (NO `todo!()`): the ssr arm does `let state = expect_context::<AppState>();` then `state.activity_cache.ask(GetRankedActivity { limit: Some(50), featured_only: false }).await` (list) / `state.activity_cache.ask(GetActivityItem(id)).await` (detail), mapping the kameo error via `ServerFnError::new(e.to_string())`.
- [ ] `crates/client/src/pages/home.rs` calls `{activity_section()}` in the `HomePage` view body and defines the `#[cfg(feature = "brick-activity")]` / `#[cfg(not(...))]` `activity_section()` pair; the strip uses `.into_iter().take(4)` against the server-ranked list (no client re-sort).
- [ ] **Public / no-auth:** the `#[server]` functions point at `GET /api/activity` and `GET /api/activity/{id}` (the *public* router), not `/api/admin/activity`; no `Authorization`/`Bearer` header is constructed anywhere in `crates/client/**`. Verify with `grep -ri "bearer\|authorization\|api-key\|admin/activity" crates/client/src` returning **no matches**.
- [ ] **Manual route smoke (run `cargo leptos serve` or the dev server):** `GET http://localhost:3000/activity` returns **HTTP 200** and the HTML body contains the string `Activity` (the `<h1>`); `GET http://localhost:3000/activity/1` returns **HTTP 200** (renders the detail page or the "Contribution Not Found" branch — both are 200, never 401/403); `GET http://localhost:3000/` returns **HTTP 200** with a `Recent Activity` section heading present once at least one activity row exists.
- [ ] **Rendered fields:** an `/activity` card shows the impact value, a forge label (`GitHub`/`Codeberg`), a state label (`Merged`/`Closed`/`Open`), the reference date (`%b %Y`), and links internally to `/activity/{id}`; the `/activity/{id}` detail page renders an outbound `<a href={item.url} target="_blank" rel="noopener noreferrer">` to the upstream PR/issue URL.

## Files likely touched

Client (this phase's core):
- `crates/client/src/pages/activity.rs` — **new** (ranked list/grid).
- `crates/client/src/pages/activity_detail.rs` — **new** (detail page, `id: i64` param).
- `crates/client/src/pages/mod.rs` — add gated `mod` + `pub use` for both pages.
- `crates/client/src/pages/home.rs` — add `{activity_section()}` call + the gated `activity_section()` pair.
- `crates/client/src/api.rs` — add `GetActivityList` + `GetActivityItemById` `#[server]` fns (real SSR bodies).
- `crates/client/src/app.rs` — add the two routes + `brick-activity` to both `app_routes()` cfg gates.
- `crates/client/Cargo.toml` — add `brick-activity = ["plinth-shared/brick-activity"]` + append to `default`.

Workspace / shared (only if not already present):
- `Cargo.toml` (root) — add `brick-activity` to `bin-features` and `lib-features` if absent.
- `crates/shared/Cargo.toml` — `brick-activity = []` leaf marker if Phase 01 has not added it.

Optional (step 8):
- `crates/shared/src/config.rs` — `ActivityPageConfig` + `PagesConfig.activity`.
- `crates/shared/src/toml_config.rs` — matching `PagesTomlConfig` field if the server projects page config.

## Pitfalls

- **Editing only one `app_routes()` cfg gate.**
  *Symptom:* either every brick route vanishes from the default build (page 404s), or
  `cargo build` errors `cannot find value ActivityPage in this scope` in the fallback
  variant. *Cause:* the route table is two whole functions selected by `#[cfg(all(...))]`
  / `#[cfg(not(all(...)))]`; adding `brick-activity` to only one expression makes the two
  cfg sets overlap or leave a gap. *Recovery:* add `feature = "brick-activity"` to **both**
  gate lists (step 7a) and put the `<Route>` lines only in the all-bricks body.

- **Bare `view!` returned from `match` arms.**
  *Symptom:* `` `match` arms have incompatible types `` / "expected `EitherOf3`...".
  *Cause:* Leptos `view!` blocks have distinct opaque types per branch. *Recovery:* wrap
  each arm in `EitherOf3::A/B/C` exactly as portfolio does (steps 4, 5, 9). Empty/list/error
  is the canonical three-way; a two-way needs `Either::Left/Right`.

- **Treating the detail param as a slug.**
  *Symptom:* `get_activity_item_by_id` never finds anything; type error binding a `String`
  where `i64` is expected. *Cause:* portfolio keys by `slug: String`; activity keys by
  `id: BIGSERIAL` (`i64`). *Recovery:* the route is `/activity/:id`, the param key is
  `"id"`, parse with `.parse::<i64>()`, and the `#[server]` fn takes `id: i64`.

- **Re-sorting on the client.**
  *Symptom:* home strip / list order disagrees with the ranking config; flicker between SSR
  and hydrate. *Cause:* the server computes `score` in SQL (`ORDER BY score DESC,
  reference_date DESC`) at read time; sorting again on the client double-orders and can
  diverge. *Recovery:* render in received order; `.take(N)` only for the strip.

- **Importing `reqwest` / building a `Bearer` header in the client.**
  *Symptom:* WASM build bloat or compile failure; or the page demands auth. *Cause:*
  copying CLI/admin patterns. *Recovery:* the client only calls the public `#[server]`
  bridge against `GET /api/activity` — no HTTP client, no token. The `grep` in the
  acceptance criteria enforces this.

- **Fabricating fields on the shared types.**
  *Symptom:* `no field html_content on ActivityItem` (or `description`, `tech_stack`).
  *Cause:* assuming activity mirrors portfolio's fields. *Recovery:* render only the fields
  in the step-1 contract (`title`, `url`, `repo_owner/name`, `number`, `forge`, `state`,
  `impact`, the three dates, `body`). If a field is genuinely missing from Phase 01's type,
  that is a Phase 01 gap to report, not a field to invent here.

- **Forgetting the workspace `bin-features` / `lib-features`.**
  *Symptom:* `cargo check` passes but `cargo leptos build` / the flake build does not
  include the activity surface (routes 404 in the real binary). *Cause:* cargo-leptos uses
  the root `Cargo.toml` feature lists, not crate defaults. *Recovery:* ensure
  `brick-activity` is in both lists (step 2) — but only add it once; another wave-2 phase
  may have already added it.

- **WASM target not installed.**
  *Symptom:* `error: target 'wasm32-unknown-unknown' not found`. *Cause:* local toolchain.
  *Recovery:* `rustup target add wasm32-unknown-unknown` (the Nix flake toolchain already
  has it).

## Reference

- **Sequencing only:** `./03-server-brick-core.md` must land first — it defines the public
  `GET /api/activity` + `GET /api/activity/{id}` handlers and the `ActivityListItem` /
  `ActivityItem` / `Forge` / `ActivityKind` / `ActivityState` shared types this phase
  consumes. `./07-feed-and-search.md` owns `/feeds/activity.xml` and the search union — do
  not implement them here. `./04-lazy-refresh-actor.md` owns the TTL refresh. None of these
  share files with this phase (client-only), so no rebase is required.
- **Live patterns copied (read for exact idiom):**
  `crates/client/src/pages/portfolio.rs` (list/grid + `EitherOf3` + `Resource::new(|| (), ...)`),
  `crates/client/src/pages/portfolio_detail.rs` (`use_params_map`, three-way detail render),
  `crates/client/src/pages/home.rs:154-229` (the `portfolio_section()` / `#[cfg(not)]` pair
  and `.take(N)` strip), `crates/client/src/api.rs:67-82` (portfolio `#[server]` fns),
  `crates/client/src/app.rs:78-121` (two-variant `app_routes()` cfg pattern),
  `crates/client/src/pages/mod.rs:17-51` (gated module + re-export pattern),
  `crates/shared/src/config.rs:107-130, 208-221` (page-config template for step 8).
- **Design brief:** Surfaces 4a (dedicated `/activity` page) + 4b (home strip) of the
  Forge Activity brick; ranking computed in SQL at read time (`score DESC, reference_date
  DESC`); reference date = `coalesce(merged_at, closed_at, created_at)`; impact `SMALLINT
  1..=10`; public reads are unauthenticated.
