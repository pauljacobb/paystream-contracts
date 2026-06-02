# PayStream Python SDK

Python SDK for PayStream Soroban contract interactions on Stellar.

## Install

```bash
pip install ./python-sdk
# or editable:
# pip install -e ./python-sdk
```

## Quickstart (read-only)

```python
from paystream_sdk import PayStreamClient
from stellar_sdk import Networks

client = PayStreamClient(
    rpc_url="https://soroban-testnet.stellar.org",
    network_passphrase=Networks.TESTNET_NETWORK_PASSPHRASE,
    contract_id="C...",  # PayStream contract id
)

stream = client.get_stream(0)
claimable = client.claimable(0)
count = client.stream_count()
```

> Note: Methods that require state-changing operations return unsigned transaction XDR (so your backend/integration can sign using your preferred key management).

## Example

See: `examples/python/stream.py`.

Set env vars before running:
- `EMPLOYER_SECRET` (S...)
- `EMPLOYEE_PUBLIC` (G...)
- `TOKEN_CONTRACT_ID` (C...)
- `STREAM_CONTRACT_ID` (C...)

