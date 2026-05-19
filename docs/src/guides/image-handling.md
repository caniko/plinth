# Image Handling

Plinth integrates with [Immich](https://immich.app) for self-hosted image management. Images are stored in Immich and served to readers through Plinth's image proxy.

## Architecture

```
Author                  Plinth CLI              Immich            Plinth Server          Browser
  |                        |                      |                    |                    |
  |-- publish post.typ --> |                      |                    |                    |
  |                        |-- upload images ---> |                    |                    |
  |                        |<-- asset IDs ------- |                    |                    |
  |                        |-- POST article ----->|                    |                    |
  |                        |   (with /api/images/{id} URLs)           |                    |
  |                        |                      |                    |                    |
  |                        |                      |     GET /api/images/{id} <------------- |
  |                        |                      |<---- fetch with API key --------------- |
  |                        |                      |---- stream bytes ---------------------->|
```

Immich is never exposed publicly. Plinth authenticates with Immich using an API key and streams the image bytes to the browser with aggressive caching.

## Server configuration

Enable the image proxy by setting the Immich URL and API key:

```toml
# plinth.toml
[immich]
api_url = "http://immich:2283"

[images]
cache_max_age = 31536000  # 1 year in seconds
```

The API key is set via environment variable (not in TOML, for security):

```bash
export IMMICH_API_KEY="your-immich-api-key"
```

Or in NixOS:

```nix
services.plinth.immich = {
  apiUrl = "http://immich:2283";
  apiKey = "your-key";  # or use secrets management
};
```

## Image proxy endpoint

```
GET /api/images/{asset_id}?size=original|preview|thumbnail
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| `asset_id` | required | Immich asset UUID |
| `size` | `original` | Image variant: `original`, `preview`, or `thumbnail` |

Response headers include `Cache-Control: public, max-age=31536000, immutable` for browser and CDN caching.

The `asset_id` is validated as a UUID to prevent path traversal.

## CLI image upload

When publishing Typst posts with local image references:

```bash
plinth-cli publish post.typ \
  --immich-url https://immich.example.com \
  --immich-api-key YOUR_KEY
```

The CLI scans for `#blog-image("local-file.jpg", ...)` references, uploads each file to Immich, and replaces the local path with `/api/images/{asset_id}` in the compiled HTML.

Environment variables `IMMICH_API_URL` and `IMMICH_API_KEY` can be used instead of flags.
