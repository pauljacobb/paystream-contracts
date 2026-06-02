# TODO - Python PayStream SDK

- [ ] Create Python SDK package scaffold under `python-sdk/`
  - [ ] Add `python-sdk/pyproject.toml`
  - [ ] Add `python-sdk/README.md`
  - [ ] Add `python-sdk/src/paystream_sdk/__init__.py`
- [x] Implement core SDK module
  - [x] `client.py` with `PayStreamClient` supporting read-only + tx-building methods
  - [x] `types.py` with `StreamStatus`, `Stream`, etc.
  - [x] `utils.py` with SCVal/xdr conversion helpers and polling

- [ ] Provide transaction submission helper(s)
  - [x] Implement `submit_transaction(signed_xdr)` (or equivalent) in client

- [ ] Add/adjust Python examples
  - [x] Update `examples/python/stream.py` to use the SDK
  - [x] Or add `examples/python/sdk_example.py`

- [ ] Sanity checks
  - [x] Python import test for `paystream_sdk`
- [x] Final run instructions in README



