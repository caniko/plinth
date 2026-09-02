---
name: blog-writer
description: >
  Write blog posts, create articles, author content for the Plinth blog system,
  suggest images for posts, format Typst or Markdown blog content, plan blog
  post structure, draft post outlines, write SEO descriptions and tags.
  Use this skill whenever the user mentions: "write a blog post", "create an article",
  "draft a post", "suggest images for", "blog content", "write in typst",
  "new post about", "blog post about", "write about", or wants help authoring,
  formatting, or planning any blog content -- even if they don't say "blog" explicitly.
---

**Cross-repository work:** If scope spans repositories, invoke `$graphify` before discovery, planning, or edits. Query an existing graph; build/update a merged graph when missing, stale, or incomplete. Reuse a current graph for the same repository set.

# Plinth Blog Writer

Write a complete Plinth post the user can save and publish with `plinth-cli publish <file>`. Typst (`.typ`) is preferred; Markdown (`.md`) is simpler. Ask if the format is unclear.

## Frontmatter

Typst uses `// ---` comment fences; Markdown uses YAML `---`. Fields:

| Field | Required | Default |
|-------|----------|---------|
| `title` | Yes | — |
| `description` | No | `""` |
| `tags` | No | `[]` |
| `author` | No | Site config author |
| `published` | No | `true` |
| `featured` | No | `false` |

```typst
// ---
// title: Your Post Title
// description: A concise meta description for SEO (1-2 sentences)
// tags: ["tag1", "tag2", "tag3"]
// author: Can Tartanoglu
// published: true
// featured: false
// ---
```

```markdown
---
title: Your Post Title
description: A concise meta description for SEO
tags: ["tag1", "tag2"]
author: Can Tartanoglu
published: true
featured: false
---
```

## Image functions (Typst)

Auto-imported from the blog template.

```typst
#blog-image("photo.jpg", placement: "inline", caption: "Optional caption", alt: "Descriptive alt text")
#hero-image("banner.jpg", caption: "Optional caption", alt: "Descriptive alt text")
#gallery(
  (src: "a.jpg", alt: "First image", caption: "Caption A"),
  (src: "b.jpg", alt: "Second image", caption: "Caption B"),
)
```

`#blog-image` placements: `"inline"` (default), `"hero"`, `"float-left"`, `"float-right"`, `"full-width"`. Gallery dicts need `src`; `alt` and `caption` are optional.

`src` may be a local path (`"photos/sunset.jpg"` — CLI uploads to Immich on publish), a proxy URL (`"/api/images/{asset-uuid}"`), or an external URL.

Markdown images are standard `![Alt text](image-url.jpg "Optional title")` with no placement controls. Use Typst for hero, float, or gallery.

## Short examples

```typst
// ---
// title: Building a Blog Engine in Rust
// description: How I built Plinth, a Leptos-powered blog with Postgres and Typst support
// tags: ["rust", "leptos", "web"]
// published: true
// featured: true
// ---

= Building a Blog Engine in Rust
#hero-image("hero-workshop.jpg", alt: "Workbench with Rust code on a screen")
Leptos compiles to WebAssembly for the client and SSR with Axum.
```

```markdown
---
title: Building a Blog Engine in Rust
description: How I built Plinth with Leptos, Postgres, and Typst
tags: ["rust", "leptos", "web"]
featured: true
---

# Building a Blog Engine in Rust
Leptos compiles to WebAssembly for the client and SSR with Axum.
```

## Image suggestions

For each suggested image give: a detailed AI prompt, 3–5 stock search phrases, and a ready-to-paste `#blog-image` / `#hero-image` / `#gallery` call with placement, caption, and alt.

Alt text: describe what the image shows, include visible key information, keep under ~125 characters when possible.

## SEO

- **description**: 120–160 characters; value, not just topic.
- **tags**: 2–5 specific lowercase tags the blog already uses.
- **title**: under 60 characters.
- Front-load the hook and important concepts (embeddings use the first 5000 characters after stripping markup). Reading time is `ceil(word_count / 200)` minutes (min 1).

## Publishing

```bash
plinth-cli publish post.typ
plinth-cli publish post.md
plinth-cli publish -i
plinth-cli init post
```

Local Typst images need `IMMICH_API_URL` and `IMMICH_API_KEY`. The CLI uploads them and rewrites paths to `/api/images/{asset_id}`. Format is by extension (`.md` / `.typ`). Slug is generated from title. Published images are served at `GET /api/images/{asset_id}?size=original|preview|thumbnail`.
