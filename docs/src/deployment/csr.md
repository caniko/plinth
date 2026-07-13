# CSR Static Build

Plinth's default deployment target is the SSR + hydrate server package. Use that
for production sites that should render HTML on the server, serve feeds, proxy
images, and expose the admin API from the same process.

The `plinth-csr` package is a client-only static build. It is useful for
serverless/static-host previews or deployments where another Plinth API server is
available. The bundle renders all routes in the browser and reads public content
with relative `GET /api/*` REST calls instead of server-only Dioxus functions.

Build it with:

```bash
nix build .#plinth-csr
```

The result is a static directory containing `index.html`, `pkg/plinth.js`,
`pkg/plinth_bg.wasm`, `pkg/plinth.css`, and copied assets from `public/`.

By default the CSR bundle assumes the API is available on the same origin as the
static files. For a separate API origin, build with `PLINTH_CSR_API_BASE` set to
the API origin, for example `https://plinth-api.example.com`. Cross-origin
deployments require the API host to allow browser CORS requests.
