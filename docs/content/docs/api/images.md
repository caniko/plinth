+++
title = "Image Proxy"
description = "Immich image proxy endpoint"
weight = 30
+++

The image proxy serves images from a private Immich instance through Plinth, adding authentication and caching.

## Endpoint

```
GET /api/images/{asset_id}?size=original|preview|thumbnail
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `asset_id` | path (UUID) | required | Immich asset ID |
| `size` | query string | `original` | Image variant |

## Size variants

| Size | Description | Immich endpoint |
|------|-------------|-----------------|
| `original` | Full-resolution image | `/api/assets/{id}/original` |
| `preview` | Resized preview | `/api/assets/{id}/thumbnail?size=preview` |
| `thumbnail` | Small thumbnail | `/api/assets/{id}/thumbnail?size=thumbnail` |

## Response headers

```
Content-Type: <forwarded from Immich>
Cache-Control: public, max-age=31536000, immutable
```

The 1-year cache duration is configurable via `images.cache_max_age` in `plinth.toml`.

## Error responses

| Status | Cause |
|--------|-------|
| 400 | `asset_id` is not a valid UUID |
| 404 | Asset not found in Immich |
| 502 | Failed to connect to Immich |
| 503 | Image proxy not configured (no `IMMICH_API_URL`) |

## Security

- The `asset_id` is validated as a UUID to prevent path traversal attacks
- Immich API key is never exposed to the client — Plinth authenticates server-side
- The endpoint is publicly accessible (images are meant to be viewed by readers)
