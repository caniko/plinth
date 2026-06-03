# plinth.toml

Plinth reads configuration from `plinth.toml` (or the path in `PLINTH_CONFIG`). All fields have defaults — an empty file produces a working configuration.

## `[site]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `"Plinth"` | Site name in header and page titles |
| `tagline` | string | `"Welcome to my website"` | Short tagline on the home page |
| `description` | string | `"A personal website"` | Default meta description |
| `lang` | string | `"en"` | HTML `lang` attribute |
| `default_theme` | string | `"dark"` | Default colour theme (`"dark"` or `"light"`) |

## `[site.author]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | `"Admin"` | Default author name for articles |
| `email` | string | `""` | Email shown in footer (empty = hidden) |

## `[site.social]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `github` | string | `""` | GitHub profile URL (empty = hidden) |
| `gitlab` | string | `""` | GitLab profile URL |
| `codeberg` | string | `""` | Codeberg profile URL |
| `mastodon` | string | `""` | Mastodon profile URL |
| `bluesky` | string | `""` | Bluesky profile URL |

## `[site.footer]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `project_name` | string | `"Plinth"` | Project name in footer attribution |
| `project_url` | string | `"https://codeberg.org/caniko/plinth"` | Project URL in footer |

## `[[site.nav]]`

Navigation items (order matters). Each entry has:

| Key | Type | Description |
|-----|------|-------------|
| `label` | string | Link text |
| `path` | string | URL path |

Default navigation:

```toml
[[site.nav]]
label = "Posts"
path = "/posts"

[[site.nav]]
label = "Projects"
path = "/projects"

[[site.nav]]
label = "About"
path = "/about"
```

## `[pages.home]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `title` | string | `""` | Home page title (empty = use site name) |
| `description` | string | `""` | Home page meta description |

## `[pages.blog]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `title` | string | `"Posts"` | Blog listing page title |
| `subtitle` | string | `""` | Subtitle below the title |
| `description` | string | `""` | Meta description |

## `[pages.portfolio]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `title` | string | `"Projects"` | Portfolio page title |
| `subtitle` | string | `""` | Subtitle below the title |
| `description` | string | `""` | Meta description |

## `[pages.about]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `title` | string | `"About Me"` | About page title |
| `description` | string | `""` | Meta description |

## `[server]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | string | `"127.0.0.1"` | Bind address |
| `port` | u16 | `3000` | Bind port |

## `[database]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `database_url` | string | `"postgres://plinth:plinth@localhost:5432/plinth"` | Postgres connection URL |

## `[observability]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `service_name` | string | `"plinth"` | OTLP service name |
| `log_level` | string | `"info"` | Rust log level (`RUST_LOG` format) |
| `otlp_endpoint` | string | `""` | OTLP endpoint URL (empty = disabled) |
| `otlp_headers` | string | `""` | OTLP auth headers (comma-separated `key=value`) |

## `[search]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_limit` | usize | `10` | Default search result count |
| `related_limit` | usize | `5` | Default related articles count |
| `min_similarity` | f32 | `0.5` | Minimum cosine similarity for opinion tracking |

## `[content]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `words_per_minute` | usize | `200` | WPM for reading time calculation |
| `vector_truncation` | usize | `5000` | Max characters before generating embeddings |

## `[feeds]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `blog_limit` | usize | `50` | Max entries in `/feeds/blog.xml` |
| `projects_limit` | usize | `50` | Max entries in `/feeds/projects.xml` |
| `activity_limit` | usize | `50` | Max entries in `/feeds/activity.xml` |

## `[ranking]`

Controls how activity entries are scored for the ranked `/activity` page, the home strip, and the public API. Score is computed at read time, so changes take effect on the next request.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strategy` | string | `"exponential"` | `"exponential"`, `"linear"`, or `"pure"` |
| `half_life_days` | float | `365.0` | Exponential ranking: `impact * 0.5 ^ (age_days / half_life_days)` |
| `window_days` | float | `730.0` | Linear ranking: `impact * max(0, 1 - age_days / window_days)` |

`age` uses the reference date `coalesce(merged_at, closed_at, created_at)`. `pure` uses `impact` alone, with the most recent reference date as a tiebreaker. Results are ordered by score descending, then reference date descending.

## `[forge]`

Controls how the server fetches and refreshes activity metadata from code forges. Tokens are provided via environment variables only, never in this file. See [Environment Variables](environment-vars.md).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `refresh_ttl_secs` | integer | `3600` | Refresh an entry in the background when its `fetched_at` is older than this many seconds |
| `refresh_backoff_secs` | integer | `900` | Minimum seconds between refresh attempts for an entry that errored |
| `github_base_url` | string | `"https://api.github.com"` | GitHub API base URL, overrideable for GitHub Enterprise or testing |
| `codeberg_base_url` | string | `"https://codeberg.org/api/v1"` | Codeberg/Forgejo API base URL, overrideable for self-hosted Forgejo or testing |

```toml
[ranking]
strategy = "exponential"
half_life_days = 365.0
window_days = 730.0

[forge]
refresh_ttl_secs = 3600
refresh_backoff_secs = 900
github_base_url = "https://api.github.com"
codeberg_base_url = "https://codeberg.org/api/v1"
# tokens via GITHUB_TOKEN / CODEBERG_TOKEN env vars, not here
```

## `[immich]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `api_url` | string | `""` | Immich server URL (empty = image proxy disabled) |

## `[images]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cache_max_age` | u64 | `31536000` | Cache-Control max-age for proxied images (seconds) |

## `[analytics]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `plausible_domain` | string | `""` | Site domain tracked by Plausible (empty = disabled) |
| `plausible_script_url` | string | `""` | URL to your Plausible script (e.g. `https://plausible.example.com/js/script.js`) |

Both fields must be set for the Plausible `<script>` tag to be injected. This keeps analytics fully opt-in.

## `[donation]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `false` | Enable donation links across the site |
| `cta_text` | string | `""` | Custom text for end-of-article CTA (default: "If you found this useful, consider supporting my work.") |

## `[[donation.links]]`

Each entry defines a donation platform link:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `platform` | string | *(required)* | Platform identifier: `"kofi"`, `"github_sponsors"`, `"liberapay"`, or `"custom"` |
| `url` | string | *(required)* | URL to your profile on the platform |
| `label` | string | `""` | Custom display label (empty = auto-generated from platform name) |

When enabled, donation links appear in three places:
- **Header**: A "Support" link with heart icon in the navigation bar
- **End of articles**: A compact CTA after blog post content
- **Footer**: A heart icon alongside social links
- **`/support` page**: A dedicated page showing all configured platforms as cards

```toml
[donation]
enabled = true
cta_text = "If you found this useful, consider supporting my work."

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/yourusername"

[[donation.links]]
platform = "github_sponsors"
url = "https://github.com/sponsors/yourusername"

[[donation.links]]
platform = "liberapay"
url = "https://liberapay.com/yourusername"
```

## Full example

```toml
[site]
name = "My Site"
tagline = "Systems, science, and software"
description = "Personal website and blog"
lang = "en"
default_theme = "dark"

[site.author]
name = "Jane Doe"
email = "jane@example.com"

[site.social]
github = "https://github.com/janedoe"
mastodon = "https://fosstodon.org/@janedoe"

[site.footer]
project_name = "Plinth"
project_url = "https://codeberg.org/caniko/plinth"

[[site.nav]]
label = "Posts"
path = "/posts"

[[site.nav]]
label = "Projects"
path = "/projects"

[[site.nav]]
label = "About"
path = "/about"

[pages.blog]
title = "Blog"
subtitle = "Notes on software and systems"

[server]
host = "127.0.0.1"
port = 3000

[database]
database_url = "postgres://plinth:plinth@localhost:5432/plinth"

[observability]
log_level = "info"

[search]
default_limit = 10

[content]
words_per_minute = 200

[feeds]
blog_limit = 50
projects_limit = 50
activity_limit = 50

[ranking]
strategy = "exponential"
half_life_days = 365.0
window_days = 730.0

[forge]
refresh_ttl_secs = 3600
refresh_backoff_secs = 900
github_base_url = "https://api.github.com"
codeberg_base_url = "https://codeberg.org/api/v1"

[analytics]
plausible_domain = "example.com"
plausible_script_url = "https://plausible.example.com/js/script.js"

[donation]
enabled = true

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/janedoe"

[[donation.links]]
platform = "github_sponsors"
url = "https://github.com/sponsors/janedoe"
```
