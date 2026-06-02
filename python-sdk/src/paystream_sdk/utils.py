from __future__ import annotations

import time
from typing import Any, Callable, Optional

from stellar_sdk import rpc, scval
from stellar_sdk.xdr import SCVal
from stellar_sdk import xdr


def _scval_to_native(val: SCVal) -> Any:
    """Best-effort conversion for simple scalar ScVal types."""
    # stellar-sdk's scValToNative helper exists in some versions.
    try:
        from stellar_sdk import scValToNative  # type: ignore

        return scValToNative(val)
    except Exception:
        # Fallback: handle a few common scalar cases.
        if isinstance(val, scval.ScVal) or hasattr(val, "type"):
            # Let xdr type drive conversion
            # NOTE: This may not cover complex maps/structs.
            try:
                return val.to_xdr_string()
            except Exception:
                return val
        return val


def poll_transaction(
    server: rpc.Server,
    tx_hash: str,
    *,
    timeout_s: int = 45,
    poll_interval_s: float = 2.0,
    on_update: Optional[Callable[[Any], None]] = None,
):
    """Polls until SUCCESS/FAILED or timeout.

    Returns the rpc get_transaction response when SUCCESS.
    """
    start = time.time()
    while True:
        status = server.get_transaction(tx_hash)
        if on_update:
            try:
                on_update(status)
            except Exception:
                pass

        if getattr(status, "status", None) == rpc.Api.GetTransactionStatus.SUCCESS:
            return status
        if getattr(status, "status", None) == rpc.Api.GetTransactionStatus.FAILED:
            raise RuntimeError(f"Transaction failed: {tx_hash}")

        if time.time() - start > timeout_s:
            raise TimeoutError(f"Transaction not confirmed after {timeout_s}s: {tx_hash}")

        time.sleep(poll_interval_s)


def require_scval_u64(v: int | str) -> SCVal:
    return scval.to_uint64(int(v))


def require_scval_i128(v: int | str) -> SCVal:
    # stellar-sdk uses i128/opaque; scval has to_int128 in some versions.
    try:
        return scval.to_int128(int(v))
    except Exception:
        # Fallback: construct via xdr helpers if available
        return scval.to_int128(int(v))

