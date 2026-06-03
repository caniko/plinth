# Curating External Activity

The **activity** feature showcases pull requests and issues you have landed on other people's repositories across GitHub and Codeberg, ranked by impact and recency. It is distinct from the portfolio, which is for your own projects.

Each entry is fetched from the forge once, at add time, by the CLI. The CLI also computes the search embedding locally; the server never runs fastembed for activity publishing. The server keeps forge metadata fresh in the background according to the `[forge]` settings in [plinth.toml](../configuration/plinth-toml.md).

## Adding a contribution

```bash
plinth activity add \
  --forge github \
  --repo owner/name \
  --pr 1234 \
  --impact 7 \
  --featured
```

For an issue, use `--issue <n>` instead of `--pr <n>`.

| Flag | Required | Description |
|------|----------|-------------|
| `--forge` | yes | `github` or `codeberg` |
| `--repo` | yes | Repository in `owner/name` form |
| `--pr <n>` | one of | Pull-request number |
| `--issue <n>` | one of | Issue number, mutually exclusive with `--pr` |
| `--impact <1-10>` | yes | Curated impact score |
| `--featured` | no | Show in the home-page strip |

On `add`, the CLI fetches the PR or issue metadata from the forge, embeds the title and body with fastembed (`AllMiniLML6V2`, 384 dimensions), and posts the result to `/api/admin/activity`.

## Updating impact or featured

`<id>` is the numeric entry id shown by `plinth activity list`.

```bash
plinth activity update <id> --impact 9 --featured true
```

At least one of `--impact` or `--featured` is required.

## Removing

```bash
plinth activity remove <id>
```

## Listing

```bash
plinth activity list
```

The list is read from the public `/api/activity` endpoint and is already ranked by the server.

## Authentication and rate limits

The CLI uses `PLINTH_API_URL` and `PLINTH_API_KEY` to publish, update, and remove entries from your Plinth instance.

Public forge data works without forge tokens, but unauthenticated requests are rate-limited. Set `GITHUB_TOKEN` or `CODEBERG_TOKEN` when fetching activity to raise GitHub's limit and reduce throttling. See [Environment Variables](../configuration/environment-vars.md).
