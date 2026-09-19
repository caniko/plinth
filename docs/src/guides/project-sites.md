# Project Sites

`plinth-project` builds static project landing sites from a
`plinth-project.toml` file. Each `[[pages]]` page renders to `{slug}/index.html`
(`index.html` for the home page), plus `style.css` and any `[[assets]]`.

```bash
plinth-project build --config website/plinth-project.toml --out public
```

## Site metadata

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | string | required | Site title, shown in the nav bar and tab |
| `description` | string | required | Fallback `<meta name="description">` |
| `base_url` | string | `"/"` | Base URL used for absolute links |
| `canonical_domain` | string | — | Canonical domain (e.g. `"example.com"`). Used for absolute sitemap URLs and the GitHub Pages `CNAME` file |
| `footer_note` | string | `""` | Text displayed in the site footer |
| `primary_person` | string | — | ID of a `[[people]]` entry shown as the site author |

## Machine-readable output

Every build also emits companions for search engines and LLM traversal:

| File | Contents |
|------|----------|
| `sitemap.xml` | One `<loc>` per page; absolute when `canonical_domain` (or an absolute `base_url`) is set, path-only otherwise |
| `robots.txt` | `Allow: /`, plus a `Sitemap:` line when the origin is known |
| `llms.txt` | Site title/description, every page (title, URL, description), and every `[[projects]]` entry with its source/demo/links |
| `projects.json` | The `[[projects]]` table verbatim as JSON |
| `CNAME` | The canonical domain, only when `canonical_domain` is set |

Set `canonical_domain` for any site published to GitHub Pages: the `CNAME`
file flows through the Pages artifact automatically, so the custom domain
survives redeploys with no workflow changes.
