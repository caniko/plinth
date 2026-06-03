# Activity API

Endpoints for curated external contributions: pull requests and issues you landed on other repositories, ranked by impact and recency.

## Admin

Admin endpoints require Bearer token authentication with `PLINTH_API_KEY`.

```
Authorization: Bearer <your-api-key>
```

### Publish activity

```
POST /api/admin/activity
Content-Type: application/json
```

Upserts by the natural key `(forge, repo_owner, repo_name, kind, number)`.

**Request body** (`PublishActivityRequest`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `forge` | string | yes | `"github"` or `"codeberg"` |
| `repo_owner` | string | yes | Repository owner |
| `repo_name` | string | yes | Repository name |
| `kind` | string | yes | `"pr"` or `"issue"` |
| `number` | integer | yes | PR or issue number, greater than `0` |
| `url` | string | yes | Canonical forge URL |
| `title` | string | yes | Contribution title |
| `body` | string | no | Contribution body or description |
| `state` | string | yes | `"open"`, `"closed"`, or `"merged"` |
| `created_at` | string | yes | ISO-8601 creation timestamp |
| `closed_at` | string | no | ISO-8601 close timestamp |
| `merged_at` | string | no | ISO-8601 merge timestamp |
| `impact` | integer | no | Curated impact score, `1..=10`, default `1` |
| `additions` | integer | no | Lines added |
| `deletions` | integer | no | Lines deleted |
| `comments_count` | integer | no | Number of comments reported by the forge |
| `labels` | string[] | no | Label names |
| `repo_stars` | integer | no | Repository star count |
| `embedding` | float[] | no | 384-dimensional fastembed vector supplied by the CLI |
| `featured` | bool | no | Show in the home strip, default `false` |
| `published` | bool | no | Include in public surfaces, default `true` |
| `content_hash` | string | no | Optional content fingerprint |

**Response** (200):

```json
{
  "success": true,
  "url": "https://github.com/owner/repo/pull/1234",
  "id": 42,
  "message": "Activity published successfully"
}
```

### Update activity

```
PATCH /api/admin/activity/{id}
Content-Type: application/json
```

Updates curated fields for a numeric activity id.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `impact` | integer | no | New impact score, `1..=10` |
| `featured` | bool | no | New featured flag |
| `published` | bool | no | New public visibility flag |

### Delete activity

```
DELETE /api/admin/activity/{id}
```

Deletes an activity entry by numeric id.

## Public

### List activity

```
GET /api/activity?limit=<n>&featured=<bool>
```

Returns entries ranked by score descending, then reference date descending. Reading a stale entry serves cached data immediately and triggers a single-flighted background refresh.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | server default | Maximum entries |
| `featured` | bool | omitted | When `true`, return featured entries only |

**Response** (200):

```json
[
  {
    "id": 42,
    "forge": "github",
    "repo_owner": "owner",
    "repo_name": "repo",
    "kind": "pr",
    "number": 1234,
    "url": "https://github.com/owner/repo/pull/1234",
    "title": "Improve build output",
    "state": "merged",
    "impact": 7,
    "labels": ["rust"],
    "featured": true,
    "score": 6.92
  }
]
```

### Get activity

```
GET /api/activity/{id}
```

Returns one activity entry by numeric id.

## Feed

```
GET /feeds/activity.xml
```

RSS 2.0 feed for curated external activity. The response is `application/rss+xml` with `Cache-Control: public, max-age=3600`; entries link to the forge URL and are ordered by ranking.

## Search

Activity entries are unioned into semantic search at `GET /api/search`. See the [Search API](./search.md).
