"""Generic fixtures for a simulated device and controllable faults."""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest

from j1939_sdk import Bus, Message, connect


@dataclass
class FaultInjector:
    active: list[tuple[str, dict[str, Any]]] = field(default_factory=list)

    def add(self, name: str, **options: Any) -> None:
        self.active.append((name, options))

    def clear(self) -> None:
        self.active.clear()


class SimulatedDevice:
    def __init__(self, bus: Bus, fault: FaultInjector) -> None:
        self.bus = bus
        self.fault = fault

    async def send(self, identifier: int, source: int, payload: bytes) -> None:
        await self.bus.send_bam(identifier, source, payload)

    async def recv(self, timeout: float = 2.0) -> Message:
        return await self.bus.recv(timeout)


@pytest.fixture
async def device(fault_injector: FaultInjector) -> Any:
    binary = os.environ.get("J1939_BAM_DEMO")
    if not binary:
        root = Path(__file__).resolve().parents[2]
        candidate = root / "target" / "debug" / "bam_demo"
        if os.name == "nt":
            candidate = candidate.with_suffix(".exe")
        binary = str(candidate)
    bus = await connect(binary)
    try:
        yield SimulatedDevice(bus, fault_injector)
    finally:
        await bus.close()


@pytest.fixture
def fault_injector() -> FaultInjector:
    return FaultInjector()
