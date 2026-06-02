from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class StreamStatus(str, Enum):
    Active = "Active"
    Paused = "Paused"
    Cancelled = "Cancelled"
    Exhausted = "Exhausted"


@dataclass(frozen=True)
class Stream:
    id: int
    employer: str
    employee: str
    token: str
    deposit: int
    withdrawn: int
    rate_per_second: int
    start_time: int
    stop_time: int
    last_withdraw_time: int
    cooldown_period: int
    status: StreamStatus
    locked: bool
    cliff_time: int
    paused_at: int

