"""
PayStream Python example — create a stream, query claimable, withdraw.

Run:
    pip install stellar-sdk
    python stream.py

Set env vars before running:
    EMPLOYER_SECRET    — employer Stellar secret key (S...)
    EMPLOYEE_PUBLIC    — employee Stellar public key (G...)
    TOKEN_CONTRACT_ID  — SEP-41 token contract ID
    STREAM_CONTRACT_ID — PayStream stream contract ID
"""

import os
import time

from stellar_sdk import Keypair, Network, SorobanServer, TransactionBuilder
from stellar_sdk.soroban_rpc import GetTransactionStatus
from stellar_sdk.xdr import SCVal
from stellar_sdk import scval

# New SDK (optional usage)
from paystream_sdk import PayStreamClient


RPC_URL = "https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE = Network.TESTNET_NETWORK_PASSPHRASE

employer = Keypair.from_secret(os.environ["EMPLOYER_SECRET"])
employee_public = os.environ["EMPLOYEE_PUBLIC"]
token_id = os.environ["TOKEN_CONTRACT_ID"]
stream_contract_id = os.environ["STREAM_CONTRACT_ID"]

server = SorobanServer(RPC_URL)


def invoke(keypair: Keypair, method: str, *args: SCVal):
    account = server.load_account(keypair.public_key)
    tx = (
        TransactionBuilder(account, NETWORK_PASSPHRASE, base_fee=100)
        .append_invoke_contract_function_op(stream_contract_id, method, list(args))
        .set_timeout(30)
        .build()
    )
    tx = server.prepare_transaction(tx)
    tx.sign(keypair)
    response = server.send_transaction(tx)

    for _ in range(10):
        time.sleep(2)
        result = server.get_transaction(response.hash)
        if result.status == GetTransactionStatus.SUCCESS:
            return result.result_value
        if result.status == GetTransactionStatus.FAILED:
            raise RuntimeError(f"{method} transaction failed")
    raise TimeoutError("Transaction not confirmed after 20s")


def simulate(method: str, *args: SCVal):
    account = server.load_account(employer.public_key)
    tx = (
        TransactionBuilder(account, NETWORK_PASSPHRASE, base_fee=100)
        .append_invoke_contract_function_op(stream_contract_id, method, list(args))
        .set_timeout(30)
        .build()
    )
    response = server.simulate_transaction(tx)
    if response.error:
        raise RuntimeError(f"Simulation error: {response.error}")
    return response.results[0].xdr


def main():
    print("Creating stream...")
    result = invoke(
        employer,
        "create_stream",
        scval.to_address(employer.public_key),
        scval.to_address(employee_public),
        scval.to_address(token_id),
        scval.to_int128(3600),   # deposit
        scval.to_int128(1),      # rate per second
        scval.to_uint64(0),      # no stop time
    )
    stream_id = scval.from_uint64(result)
    print(f"Stream ID: {stream_id}")

    stream_xdr = simulate("get_stream", scval.to_uint64(stream_id))
    print(f"Stream XDR: {stream_xdr}")

    claimable_xdr = simulate("claimable", scval.to_uint64(stream_id))
    print(f"Claimable XDR: {claimable_xdr}")


if __name__ == "__main__":
    main()
