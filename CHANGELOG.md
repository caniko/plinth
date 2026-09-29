# Changelog

## [Unreleased]

### Added

- Project-grid sections with descriptions and links drawn from the site's project references.
- Static-site sitemap, robots policy, `llms.txt`, project JSON index, and optional canonical-domain `CNAME` output.
- A check preventing retired forge hosts from returning to the Nix inputs.

### Changed

- Update Dioxus and its CLI contract to 0.7.10.
- Consolidate CI on GitHub Actions with aggregate workspace checks, Pages deployment, and trusted Attic publication.
- Use the repository's Rust toolchain file through Harbor and complete the Harbor lockfile input closure.
- Fetch Tartan UI from GitHub at the existing Dioxus-compatible revision.
- Share forge response handling, cache expiry, feed construction, and transactional tag helpers.
- Keep generated dependency, build, graph, and browser-test state out of source control.

### Fixed

- Set the project site's canonical domain so deployed sitemap and robots metadata use absolute public URLs.
- Fetch the historical transitive Harbor pin over HTTPS so hosted runners can archive the complete flake without SSH credentials.
- Allow private, build-scoped compiler-cache fallback on unmanaged build hosts.
- Preserve the Pages artifact link and isolate generated workflow cancellation groups.
- Prevent early pipe termination from masking a retired-host match in the Nix input check.
