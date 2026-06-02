"""PayStream Python SDK example — create a stream, query claimable.

Run:
  pip install ./python-sdk
  python sdk_example.py

Set env vars before running:
  EMPLOYER_SECRET     — employer Stellar secret key (S...)
  EMPLOYEE_PUBLIC     — employee Stellar public key (G...)
  TOKEN_CONTRACT_ID   — SEP-41 token contract ID
  STREAM_CONTRACT_ID  — PayStream stream contract ID

Note:
  This example signs and submits using employer keypair.
  For backend integrations you may prefer returning unsigned XDR from the SDK and signing elsewhere.
"""

import os
import time

from stellar_sdk import Keypair, Network, SorobanServer
from stellar_sdk.soroban_rpc import GetTransactionStatus

from paystream_sdk import PayStreamClient


RPC_URL = "https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE = Network.TESTNET_NETWORK_PASSPHRASE


def sign_xdr_and_submit(server: SorobanServer, keypair: Keypair, unsigned_xdr: str) -> str:
    # stellar-sdk provides sign/send helpers; keep as simple as possible.
    tx = server.transaction_from_xdr(unsigned_xdr, NETWORK_PASSPHRASE)
    tx.sign(keypair)
    result = server.send_transaction(tx)
    if result.status.name == "ERROR" or getattr(result, "error", None):
        raise RuntimeError(f"Send failed: {getattr(result, 'error', result)}")
    return result.hash


def wait_for_confirmation(server: SorobanServer, tx_hash: str, *, timeout_s: int = 60) -> None:
    start = time.time()
    while True:
        res = server.get_transaction(tx_hash)
        if res.status == GetTransactionStatus.SUCCESS:
            return
        if res.status == GetTransactionStatus.FAILED:
            raise RuntimeError(f"Transaction failed: {tx_hash}")
        if time.time() - start > timeout_s:
            raise TimeoutError(f"Timeout waiting for: {tx_hash}")
        time.sleep(2)


def main() -> None:
    employer = Keypair.from_secret(os.environ["EMPLOYER_SECRET"])
    employee_public = os.environ["EMPLOYEE_PUBLIC"]
    token_id = os.environ["TOKEN_CONTRACT_ID"]
    stream_contract_id = os.environ["STREAM_CONTRACT_ID"]

    client = PayStreamClient(
        rpc_url=RPC_URL,
        network_passphrase=NETWORK_PASSPHRASE,
        contract_id=stream_contract_id,
    )

    server = SorobanServer(RPC_URL)

    print("Creating stream...")
    unsigned_xdr = client.create_stream(
        employer=employer.public_key,
        employee=employee_public,
        token_address=token_id,
        deposit=3600,
        rate_per_second=1,
        stop_time=0,
        cooldown_period=0,
        cliff_time=0,
    )

    tx_hash = sign_xdr_and_submit(server, employer, unsigned_xdr)
    wait_for_confirmation(server, tx_hash)
    print("Submitted tx:", tx_hash)

    # For simplicity, this example does not extract the new stream id from return events.
    # Read-only methods can be called once you know STREAM_ID.
    stream_id = 0
    claimable = client.claimable(stream_id)
    print("Claimable now (stream_id=0):", claimable)


if __name__ == "__main__":
    main()

