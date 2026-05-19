# Publishing Blog Posts

Plinth supports two authoring formats: **Markdown** and **Typst**. The CLI detects the format by file extension (`.md` or `.typ`).

## Markdown workflow

### Frontmatter

Use YAML frontmatter at the top of your `.md` file:

```markdown
---
title: My Post Title
tags: ["rust", "leptos"]
description: A brief description for search engines
author: Jane Doe
published: true
featured: false
---

# Your content here

Regular Markdown with headings, code blocks, links, and images.
```

All frontmatter fields are optional. If `title` is omitted, you must provide it via the CLI or API.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | string | required | Article title |
| `tags` | string[] | `[]` | Tag names |
| `description` | string | — | Meta description |
| `author` | string | site author | Author name |
| `published` | bool | `true` | Whether the post is visible |
| `featured` | bool | `false` | Whether to feature the post |

### Publishing

```bash
plinth-cli publish article.md
```

Or during development:

```bash
cargo run --package plinth-cli -- publish article.md
```

The CLI:
1. Parses YAML frontmatter
2. Converts Markdown to HTML (pulldown-cmark)
3. Generates a 384-dim fastembed vector embedding from the text
4. Sends `POST /api/admin/articles` with content, HTML, metadata, and embedding

## Typst workflow

Typst files (`.typ`) offer richer formatting with programmatic layout capabilities.

### Frontmatter

Typst uses comment-based YAML frontmatter:

```typst
// ---
// title: My Typst Post
// tags: ["typst", "rust"]
// description: A post authored in Typst
// ---

= Introduction

This is a Typst document with rich formatting.
```

### Image functions

The Plinth Typst template (`templates/blog.typ`) provides image layout functions:

```typst
#blog-image("photo.jpg", placement: "inline", caption: "A photo", alt: "Description")
```

**Placements**: `inline`, `hero`, `float-left`, `float-right`, `full-width`

**Convenience functions**:
```typst
#hero-image("banner.jpg", alt: "Banner image")
#gallery((src: "a.jpg", alt: "First"), (src: "b.jpg", alt: "Second"))
```

### Publishing with images

When your `.typ` file references local images, the CLI handles upload:

```bash
plinth-cli publish post.typ --immich-url https://immich.example.com --immich-api-key KEY
```

The CLI:
1. Extracts comment-based YAML frontmatter
2. Scans for local image references (`#blog-image("local.jpg", ...)`)
3. Uploads local images to Immich, receives asset IDs
4. Replaces local paths with `/api/images/{asset_id}` proxy URLs
5. Compiles Typst to HTML via `typst-as-lib` + `typst-html`
6. Generates a fastembed embedding from the text content
7. Sends pre-rendered HTML + metadata to the server API

## Managing tags

```bash
# List all tags
plinth-cli tag list

# Add a tag to a post
plinth-cli tag add --post my-post-slug --tag "new-tag"

# Remove a tag from a post
plinth-cli tag remove --post my-post-slug --tag "old-tag"
```

## Managing site content

Update content blocks (e.g. the about page) via the CLI:

```bash
plinth-cli content update --key about --file about.md
```
