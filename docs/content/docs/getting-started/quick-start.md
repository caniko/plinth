+++
title = "Quick Start"
description = "Get a Plinth site running and publish your first post"
weight = 20
+++

This guide walks through getting Plinth running locally and publishing your first blog post.

## 1. Start the server

```bash
git clone https://codeberg.org/caniko/plinth.git
cd plinth
nix develop
cargo leptos watch
```

Open `http://127.0.0.1:3000` in your browser. You should see the default Plinth homepage.

## 2. Customise your site

Create or edit `plinth.toml` in the project root:

```toml
[site]
name = "My Site"
tagline = "Welcome to my corner of the internet"

[site.author]
name = "Your Name"
email = "you@example.com"
```

Restart the server to pick up changes.

## 3. Publish a blog post

Create a Markdown file `my-first-post.md`:

```markdown
---
title: My First Post
tags: ["hello", "plinth"]
description: Getting started with Plinth
---

# Hello, world!

This is my first blog post on Plinth.
```

Publish it using the CLI:

```bash
cargo run --package plinth-cli -- publish my-first-post.md
```

The CLI parses frontmatter, generates a vector embedding for semantic search, and sends the article to the server API.

## 4. View your post

Navigate to `http://127.0.0.1:3000/posts` to see your published post.

## Next steps

- [Configure your site](/docs/configuration/plinth-toml/) with all available options
- [Learn about Typst posts](/docs/guides/publishing/#typst-workflow) for richer authoring
- [Set up image hosting](/docs/guides/image-handling/) with Immich
- [Deploy to production](/docs/deployment/nixos-module/) on NixOS
