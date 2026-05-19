# Admin API

All admin endpoints are protected by Bearer token authentication. Set the token via `PLINTH_API_KEY`.

```
Authorization: Bearer <your-api-key>
```

## Publish article

```
POST /api/admin/articles
Content-Type: application/json
```

**Request body** (`PublishArticleRequest`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | no | Article title (can come from frontmatter) |
| `slug` | string | no | URL slug (auto-generated from title if omitted) |
| `description` | string | no | Meta description |
| `content` | string | yes | Markdown or Typst source content |
| `html_content` | string | no | Pre-rendered HTML (required for Typst) |
| `tags` | string[] | no | Tag names |
| `author` | string | no | Author name (defaults to site author) |
| `published` | bool | no | Visibility (default: `true`) |
| `featured` | bool | no | Featured flag (default: `false`) |
| `embedding` | float[] | no | 384-dim fastembed vector |
| `content_format` | string | no | `"Markdown"` (default) or `"Typst"` |

**Response** (200):

```json
{
  "success": true,
  "slug": "my-article",
  "id": "blog_posts:abc123",
  "message": "Article 'My Article' published successfully"
}
```

**Error** (400):

```json
{
  "error": "Title is required",
  "details": "Provide title in request or frontmatter"
}
```

## List tags

```
GET /api/admin/tags
```

Returns all tags with metadata.

**Response** (200):

```json
[
  { "id": "tags:abc", "name": "rust", "slug": "rust", "post_count": 5 }
]
```

## Add tag to post

```
POST /api/admin/posts/{post_slug}/tags
Content-Type: application/json
```

**Request body** (`AddTagRequest`):

```json
{ "tag": "new-tag" }
```

Creates the tag if it doesn't exist, then creates a `tagged` graph relation.

## Remove tag from post

```
DELETE /api/admin/posts/{post_slug}/tags/{tag_slug}
```

Removes the `tagged` graph relation. Does not delete the tag itself.

## Update site content

```
PUT /api/admin/content/{key}
Content-Type: application/json
```

**Request body** (`UpdateSiteContentRequest`):

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Content block title |
| `content` | string | Markdown source |
| `html_content` | string | Rendered HTML |

Upserts the content block — deletes existing entry with the same key and creates a new one.

## Get site content

```
GET /api/admin/content/{key}
```

Returns the site content block, or `null` if not found.
