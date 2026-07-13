# Quick Start

This guide walks through getting Plinth running locally and publishing your first blog post.

## 1. Start the server

```bash
git clone https://codeberg.org/caniko/plinth.git
cd plinth
nix develop
./scripts/dev-db.sh start
dx serve --web --fullstack
```

The `dev-db.sh` script starts a local Postgres cluster with pgvector and creates the `plinth` database. The dev shell exports `DATABASE_URL` automatically.

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

- [Configure your site](../configuration/plinth-toml.md) with all available options
- [Learn about Typst posts](../guides/publishing.md#typst-workflow) for richer authoring
- [Set up image hosting](../guides/image-handling.md) with Immich
- [Deploy to production](../deployment/nixos-module.md) on NixOS
