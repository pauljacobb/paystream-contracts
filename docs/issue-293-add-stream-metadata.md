Issue #293 — Add stream metadata field

Summary
-------
This change ensures streams include an arbitrary `metadata` string (max 256 chars) supplied by the employer.

Acceptance criteria mapping
- `metadata` parameter in `create_stream` (contracts/stream/src/lib.rs)
- Length validated to 256 chars (contracts/stream/src/lib.rs and create_streams_batch)
- Stored on `Stream` and returned by `get_stream` (contracts/stream/src/types.rs, get_stream in lib.rs)
- Updatable by employer via `update_metadata` (contracts/stream/src/lib.rs)
- Emitted in `StreamCreated` event (contracts/stream/src/events.rs)

Files touched
- contracts/stream/src/lib.rs
- contracts/stream/src/types.rs
- contracts/stream/src/events.rs

Notes
- All checks and functions already exist in the codebase. I created this file to document the implementation for reviewers and link to the issue.
