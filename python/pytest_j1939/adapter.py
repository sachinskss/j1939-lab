"""Adapter fixture from the protocol-neutral plugin contract to the SDK."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import pytest
from j1939_sdk import Bus as SdkBus, Message
from j1939_sdk import connect


class SdkBusAdapter:
    def __init__(self, bus: SdkBus) -> None:
        self._bus = bus

    async def send(
        self, identifier: int, source: int, payload: bytes, *, fault: dict[str, Any] | None = None
    ) -> None:
        await self._bus.send_bam(identifier, source, payload, fault=fault)

    async def recv(self, timeout: float = 2.0) -> Message:
        return await self._bus.recv(timeout)

    async def close(self) -> None:
        await self._bus.close()


@pytest.fixture
async def bus() -> Any:
    binary = os.environ.get("J1939_BAM_DEMO")
    if not binary:
        root = Path(__file__).resolve().parents[2]
        candidate = root / "target" / "debug" / "bam_demo"
        if os.name == "nt":
            candidate = candidate.with_suffix(".exe")
        binary = str(candidate)
    sdk_bus = await connect(binary)
    try:
        yield SdkBusAdapter(sdk_bus)
    finally:
        await sdk_bus.close()
