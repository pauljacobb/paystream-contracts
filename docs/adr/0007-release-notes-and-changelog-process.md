# ADR 0007: Define release notes + CHANGELOG process

## Status

Proposed

## Context

We maintain `CHANGELOG.md` and want consistent, automated, and auditable release notes.

The repository uses **git-cliff** (`cliff.toml`) with **Conventional Commits**, generating entries under:

- `[Unreleased]`
- Version sections (tag-based)

We also want a repeatable manual process for when automation is not available or when additional release narrative is needed.

## Decision

Adopt the following process:

### 1) PRs must use Conventional Commits

- Use commit types:
  - `feat`, `fix`, `docs`, `perf`, `refactor`, `test`, `chore`, `ci`, `revert`
- Optional scopes should describe the area (e.g., `frontend`, `backend`, `docs`, `contract/stream`).

### 2) Update `CHANGELOG.md` only through git-cliff (preferred)

- Changes are reflected by Conventional Commit messages.
- Release cut updates `CHANGELOG.md` by generating new sections from commits since the last tag.

### 3) `[Unreleased]` is generated continuously

- During development, `[Unreleased]` reflects merged PRs targeting main.
- For doc-only PRs, the changelog entry type should remain `Documentation`.

### 4) Manual “Release notes” narrative is optional but supported

- For major releases, create a short release note section (breaking changes, migrations, operational notes).
- Keep it in `CHANGELOG.md` under the version heading.

### 5) Release readiness checklist

Before tagging a release:

- CI is green
- No open security-critical issues
- Contract interface changes are documented in `docs/upgrade-guide.md` if applicable
- Frontend changes include build + CDN instructions if the delivery pipeline changed

## Consequences

- Requires contributors to follow Conventional Commits.
- Ensures deterministic changelog generation and makes release notes traceable to commits.

