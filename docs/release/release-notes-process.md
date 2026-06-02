# Release notes & CHANGELOG process

This document defines how PayStream generates `CHANGELOG.md` and how release notes are produced.

## 1. Source of truth: Conventional Commits

PayStream uses **git-cliff** (`cliff.toml`) with `conventional_commits = true`.

PRs should be merged with commit messages following Conventional Commits, e.g.:

- `feat(frontend): add stream dashboard view`
- `fix(contract/stream): prevent overflow in claimable calculation`
- `docs: update integration guide for TypeScript SDK usage`

git-cliff groups entries based on commit type and creates:

- `## [Unreleased]` during development
- `## [x.y.z] - YYYY-MM-DD` after version/tag cuts

## 2. CHANGELOG update rules

- Prefer regenerating `CHANGELOG.md` using git-cliff rather than editing it by hand.
- If manual narrative text is needed for a release, add it under the version heading.

## 3. Suggested release note structure

Under a version heading, include (when applicable):

- **Added**: new features
- **Changed**: behavior changes
- **Fixed**: bug fixes
- **Security**: security-relevant changes
- **Docs**: documentation and guides

## 4. Release readiness checklist

Before creating a release tag:

- CI is passing on `main`
- Contract upgrades are documented (see `docs/upgrade-guide.md`)
- Operational impact documented (see `docs/runbooks/`)
- Frontend delivery docs (CDN/static assets) updated if build/deploy changes


