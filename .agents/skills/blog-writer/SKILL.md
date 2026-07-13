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

**Cross-repository work:** As soon as work is known to span more than one Git repository, invoke `$graphify` before further discovery, planning, or edits. Query a relevant existing graph first; build or update a merged graph if none exists, it is stale, or it does not cover every repository in scope. Reuse a current graph already produced for the same repository set.

# Plinth Blog Writer

Write blog posts for Plinth, a Rust/Leptos personal blog platform. Posts are authored in **Typst** (preferred — richer formatting) or **Markdown** (simpler). Publish via CLI: `plinth-cli publish <file>`.

When asked to write a blog post, produce a **complete file** the user can save and publish directly. Ask which format they prefer if unclear — default to Typst.

---

## Typst Format (.typ) — PREFERRED

### Frontmatter

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

| Field | Required | Default |
|-------|----------|---------|
| `title` | Yes | — |
| `description` | No | `""` |
| `tags` | No | `[]` |
| `author` | No | Site config author |
| `published` | No | `true` |
| `featured` | No | `false` |

### Headings

```typst
= Top-level heading (H1 — use for the post title)
== Section heading (H2)
=== Subsection heading (H3)
```

### Text Formatting

```typst
*bold text*
_italic text_
`inline code`
#link("https://example.com")[link text]
- bullet list item
+ numbered list item
```

### Code Blocks

````typst
```rust
fn main() {
    println!("Hello");
}
```
````

### Image Functions

These are auto-imported from the blog template. Three functions are available:

#### `#blog-image` — single image with placement control

```typst
#blog-image("photo.jpg", placement: "inline", caption: "Optional caption", alt: "Descriptive alt text")
```

Placements:
- `"inline"` (default) — flows with text
- `"hero"` — full-width banner, use at top of post
- `"float-left"` — floats left, text wraps right
- `"float-right"` — floats right, text wraps left
- `"full-width"` — spans full content width

#### `#hero-image` — convenience for hero placement

```typst
#hero-image("banner.jpg", caption: "Optional caption", alt: "Descriptive alt text")
```

#### `#gallery` — grid of images

```typst
#gallery(
  (src: "a.jpg", alt: "First image", caption: "Caption A"),
  (src: "b.jpg", alt: "Second image", caption: "Caption B"),
)
```

Each image dict supports: `src` (required), `alt` (optional), `caption` (optional).

#### Image Sources

The `src` parameter accepts three types:
- **Local file path** — `"photos/sunset.jpg"` — CLI uploads to Immich during publish, replaces with proxy URL
- **Proxy URL** — `"/api/images/{asset-uuid}"` — already-uploaded Immich asset
- **External URL** — `"https://example.com/image.png"` — used as-is

### Complete Typst Example

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

When I started building my personal site, I wanted something that felt _native_ to the Rust ecosystem. No JavaScript build tools, no Node.js runtime — just Rust from server to browser.

== Why Leptos?

Leptos compiles to WebAssembly for the client and runs server-side rendering with Axum. This means:

- Full-stack Rust with a single language
- Server functions that call Postgres-backed APIs directly
- Hydration without a separate API layer

```rust
#[component]
fn BlogPost(slug: String) -> impl IntoView {
    let post = create_resource(move || slug.clone(), fetch_post);
    view! { /* ... */ }
}
```

== Adding Typst Support

The real game-changer was adding Typst as an authoring format.

#blog-image("typst-demo.png", placement: "full-width", caption: "Typst source alongside rendered output", alt: "Side-by-side comparison of Typst source code and HTML output")

== What I Learned

Building your own tools teaches you things no framework can. The constraints of explicit SQL migrations forced better data modeling. The Nix sandbox forced proper dependency management.
```

---

## Markdown Format (.md)

### Frontmatter

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

Same fields and defaults as Typst.

### Supported Extensions

| Extension | Syntax | Example |
|-----------|--------|---------|
| Strikethrough | `~~text~~` | ~~deleted~~ |
| Tables | GFM pipe syntax | `\| Col \| Col \|` |
| Footnotes | `[^1]` + definition | `[^1]: Footnote text` |
| Task lists | `- [x]` / `- [ ]` | Checkboxes |
| Heading attributes | `{#id .class}` | `## Title {#my-id}` |

Standard markdown (bold, italic, code blocks with language, links, images, blockquotes, lists) all work.

### Images in Markdown

Standard syntax only — no placement controls:

```markdown
![Alt text](image-url.jpg "Optional title")
```

Use Typst if you need hero, float, or gallery layouts.

### Complete Markdown Example

```markdown
---
title: Building a Blog Engine in Rust
description: How I built Plinth with Leptos, Postgres, and Typst
tags: ["rust", "leptos", "web"]
featured: true
---

# Building a Blog Engine in Rust

When I started building my personal site, I wanted something native to the Rust ecosystem.

## Why Leptos?

Leptos compiles to WebAssembly for the client and runs SSR with Axum:

- Full-stack Rust
- Server functions calling Postgres-backed APIs directly
- Hydration without a separate API

## What I Learned

Building your own tools teaches you things no framework can[^1].

| Feature | Benefit |
|---------|---------|
| Leptos SSR | Fast first paint |
| Postgres | Durable relational storage |
| Typst | Programmable layouts |

[^1]: Especially when the Nix sandbox won't let you download anything at build time.
```

---

## Image Suggestions

When suggesting images for a blog post, provide **all three** of these for each image:

### 1. AI Generation Prompt
A detailed, specific prompt for DALL-E / Midjourney / Stable Diffusion. Include style, composition, subject, mood, and what to exclude.

### 2. Stock Photo Search Terms
3-5 keyword phrases for Unsplash, Pexels, or similar.

### 3. Ready-to-Paste Typst Call
The complete `#blog-image`, `#hero-image`, or `#gallery` call with appropriate placement, caption, and alt text. Use a descriptive filename placeholder.

### Example

> **Suggested image for a "Why Leptos" section:**
>
> **AI prompt:** "Clean minimal illustration of a Rust gear logo connected to a web browser icon with WebAssembly bytecode flowing between them, dark background, technical diagram style, flat design, no text"
>
> **Stock search:** "rust programming", "webassembly diagram", "full-stack web development", "code compilation flow"
>
> ```typst
> #blog-image("leptos-wasm-flow.png", placement: "inline", caption: "Leptos compiles Rust to both server and client targets", alt: "Diagram showing Rust code compiling to a server binary and WebAssembly for the browser")
> ```

### Alt Text Guidelines

Always write meaningful alt text:
- Describe what the image shows, not what it is ("Flowchart of the build pipeline" not "diagram")
- Include key information visible in the image
- Keep under ~125 characters when possible
- For decorative images, still describe them briefly

---

## SEO & Content Best Practices

- **description**: 120-160 characters. Summarize the post's value, not just its topic. Good: "Learn how to build a full-stack Rust blog with Leptos SSR and Postgres -- from zero to deployed." Bad: "A post about Rust."
- **tags**: 2-5 specific, lowercase tags. Prefer tags the blog already uses. Avoid generic tags like "programming".
- **title**: Clear, specific, benefit-oriented. Under 60 characters for search results.
- **First paragraph**: Hook the reader. State the problem or outcome immediately.
- **Reading time**: System calculates `ceil(word_count / 200)` minutes (min 1). A 1000-word post = 5 min read.
- **Embeddings**: The first 5000 characters (after stripping markup) are used for semantic search vectors. Front-load important concepts.

---

## Publishing

After writing the file, give the user the publish command:

```bash
# Publish a Typst or Markdown post
plinth-cli publish post.typ
plinth-cli publish post.md

# Interactive mode (prompts for metadata, opens editor)
plinth-cli publish -i

# Scaffold a new post from template
plinth-cli init post
```

For Typst posts with **local images**, the user needs Immich configured:
- `IMMICH_API_URL` — Immich server URL
- `IMMICH_API_KEY` — Immich API key

The CLI uploads local images to Immich during publish and replaces paths with `/api/images/{asset_id}` proxy URLs automatically.

---

## Technical Notes

- **Slug generation**: Auto-generated from title. "Hello World" → `hello-world`. Special characters become hyphens.
- **Image proxy**: Published images are served via `GET /api/images/{asset_id}?size=original|preview|thumbnail` with 1-year cache headers.
- **Content format detection**: By file extension — `.md` = Markdown, `.typ` = Typst.
