from __future__ import annotations

from typing import List

from stellar_sdk import Address, Contract, TransactionBuilder, rpc, scval
from stellar_sdk.xdr import SCVal
from stellar_sdk import BASE_FEE


from .types import Stream, StreamStatus

from .utils import poll_transaction


TIMEOUT_SECONDS = 30


class PayStreamClient:
    """PayStream Soroban contract client.

    - Read-only calls are performed via `simulate_transaction`.
    - Mutating calls return **unsigned transaction XDR** (string) so the caller
      can sign with their own key management.
    """

    def __init__(self, *, rpc_url: str, network_passphrase: str, contract_id: str):
        self._rpc = rpc.Server(rpc_url, allow_http=True)
        self._network_passphrase = network_passphrase
        self._contract = Contract(contract_id)
        self._contract_id = contract_id

        # A well-known account used for read-only sims (mirrors TS SDK).
        self._sim_account = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"

    # ─── internals ──────────────────────────────────────────────────────────

    def _simulate_read(self, method: str, args: List[SCVal]) -> SCVal:
        account = self._rpc.get_account(self._sim_account)
        tx = (
            TransactionBuilder(
                account,
                {
                    "fee": BASE_FEE,
                    "networkPassphrase": self._network_passphrase,
                },
            )
            .add_operation(self._contract.call(method, *args))
            .set_timeout(TIMEOUT_SECONDS)
            .build()
        )
        sim = self._rpc.simulate_transaction(tx)
        # stellar-sdk uses rpc.Api.isSimulationError in TS; for py the exception model differs.
        if getattr(sim, "error", None):
            raise RuntimeError(f"Simulation failed: {sim.error}")

        # Attempt to extract retval (stellar-sdk versions vary slightly)
        result = getattr(sim, "result", None)
        if result and getattr(result, "retval", None) is not None:
            return result.retval

        # Some versions return `results[0].xdr` or similar.
        results = getattr(sim, "results", None)
        if results:
            r0 = results[0]
            if hasattr(r0, "xdr"):
                return r0.xdr
            if hasattr(r0, "retval"):
                return r0.retval

        # Last resort: return retval-like property if present.
        if hasattr(sim, "retval"):
            return sim.retval

        raise RuntimeError("Unexpected simulation response shape")


    def _build_tx(
        self,
        caller_public_key: str,
        method: str,
        args: List[SCVal],
    ) -> str:

        account = self._rpc.get_account(caller_public_key)
        tx = (
            TransactionBuilder(
                account,
                {
                    "fee": BASE_FEE,
                    "networkPassphrase": self._network_passphrase,
                },
            )
            .add_operation(self._contract.call(method, *args))
            .set_timeout(TIMEOUT_SECONDS)
            .build()
        )

        # For Soroban, buildTx should include `assembleTransaction` semantics.
        # stellar-sdk has `prepare_transaction` for signing flow; we need XDR.
        prepared = self._rpc.prepare_transaction(tx)
        # Return prepared tx XDR but keep it unsigned.
        # In stellar-sdk, prepared is usually a Transaction object; if so, to_xdr_string exists.
        if hasattr(prepared, "to_xdr"):
            return prepared.to_xdr()
        if hasattr(prepared, "to_xdr_string"):
            return prepared.to_xdr_string()
        if hasattr(prepared, "to_xdr_base64"):
            return prepared.to_xdr_base64()

        # Last resort: attempt to serialize via to_xdr_string
        return str(prepared)

    # ─── submit helpers ─────────────────────────────────────────────────────

    def submit_transaction(self, *, signed_xdr: str, poll: bool = True) -> str:
        tx = TransactionBuilder.from_xdr(signed_xdr, self._network_passphrase)
        send = self._rpc.send_transaction(tx)

        if getattr(send, "status", None) == "ERROR":
            raise RuntimeError(f"Submit failed: {getattr(send, 'error_result', send)}")

        tx_hash = send.hash
        if not poll:
            return tx_hash

        poll_transaction(self._rpc, tx_hash)
        return tx_hash

    # ─── read-only ──────────────────────────────────────────────────────────

    def get_stream(self, stream_id: int) -> Stream:
        val = self._simulate_read(
            "get_stream",
            [scval.to_uint64(int(stream_id))],
        )

        # Parse retval. stellar-sdk returns native python structures in some versions.
        # We support both dict-like and tuple-like results.
        # Expected contract struct fields (per TS Stream):
        # id, employer, employee, token, deposit, withdrawn, rate_per_second,
        # start_time, stop_time, last_withdraw_time, cooldown_period, status,
        # locked, cliff_time, paused_at
        native = getattr(val, "to_dict", None)
        if callable(native):
            data = val.to_dict()
        else:
            data = val

        # Best-effort: handle dict with keys.
        if isinstance(data, dict):
            status = StreamStatus(data["status"])
            return Stream(
                id=int(data["id"]),
                employer=str(data["employer"]),
                employee=str(data["employee"]),
                token=str(data["token"]),
                deposit=int(data["deposit"]),
                withdrawn=int(data["withdrawn"]),
                rate_per_second=int(data["rate_per_second"]),
                start_time=int(data["start_time"]),
                stop_time=int(data["stop_time"]),
                last_withdraw_time=int(data["last_withdraw_time"]),
                cooldown_period=int(data["cooldown_period"]),
                status=status,
                locked=bool(data["locked"]),
                cliff_time=int(data["cliff_time"]),
                paused_at=int(data["paused_at"]),
            )

        # Fallback: unknown shape
        raise RuntimeError("Unable to parse get_stream simulation result")

    def claimable(self, stream_id: int) -> int:
        val = self._simulate_read(
            "claimable",
            [scval.to_uint64(int(stream_id))],
        )
        # scalar
        try:
            native = scval.scval_to_native(val)  # type: ignore
            return int(native)
        except Exception:
            return int(getattr(val, "value", val))

    def claimable_at(self, stream_id: int, timestamp: int) -> int:
        val = self._simulate_read(
            "claimable_at",
            [
                scval.to_uint64(int(stream_id)),
                scval.to_uint64(int(timestamp)),
            ],
        )
        try:
            native = scval.scval_to_native(val)  # type: ignore
            return int(native)
        except Exception:
            return int(getattr(val, "value", val))

    def stream_count(self) -> int:
        val = self._simulate_read("stream_count", [])
        try:
            native = scval.scval_to_native(val)  # type: ignore
            return int(native)
        except Exception:
            return int(getattr(val, "value", val))

    # ─── mutating (return unsigned tx XDR) ──────────────────────────────────

    def initialize(self, admin: str) -> str:
        return self._build_tx(admin, "initialize", [Address(admin).to_scval()])

    def create_stream(
        self,
        employer: str,
        employee: str,
        token_address: str,
        deposit: int,
        rate_per_second: int,
        stop_time: int,
        cooldown_period: int,
        cliff_time: int,
    ) -> str:
        return self._build_tx(
            employer,
            "create_stream",
            [
                Address(employer).to_scval(),
                Address(employee).to_scval(),
                Address(token_address).to_scval(),
                scval.to_int128(int(deposit)),
                scval.to_int128(int(rate_per_second)),
                scval.to_uint64(int(stop_time)),
                scval.to_uint64(int(cooldown_period)),
                scval.to_uint64(int(cliff_time)),
            ],
        )

    def withdraw(self, employee: str, stream_id: int) -> str:
        return self._build_tx(
            employee,
            "withdraw",
            [Address(employee).to_scval(), scval.to_uint64(int(stream_id))],
        )

    def top_up(self, employer: str, stream_id: int, amount: int) -> str:
        return self._build_tx(
            employer,
            "top_up",
            [
                Address(employer).to_scval(),
                scval.to_uint64(int(stream_id)),
                scval.to_int128(int(amount)),
            ],
        )

    def update_rate(self, employer: str, stream_id: int, new_rate: int) -> str:
        return self._build_tx(
            employer,
            "update_rate",
            [
                Address(employer).to_scval(),
                scval.to_uint64(int(stream_id)),
                scval.to_int128(int(new_rate)),
            ],
        )

    def pause_stream(self, employer: str, stream_id: int) -> str:
        return self._build_tx(
            employer,
            "pause_stream",
            [Address(employer).to_scval(), scval.to_uint64(int(stream_id))],
        )

    def resume_stream(self, employer: str, stream_id: int) -> str:
        return self._build_tx(
            employer,
            "resume_stream",
            [Address(employer).to_scval(), scval.to_uint64(int(stream_id))],
        )

    def cancel_stream(self, employer: str, stream_id: int) -> str:
        return self._build_tx(
            employer,
            "cancel_stream",
            [Address(employer).to_scval(), scval.to_uint64(int(stream_id))],
        )

