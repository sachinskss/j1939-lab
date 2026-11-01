"""Protocol-neutral fixtures for devices that send and receive messages."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol

import pytest


class Bus(Protocol):
    """Small adapter contract; concrete transports live outside this plugin."""

    async def send(
        self, identifier: int, source: int, payload: bytes, *, fault: dict[str, Any] | None = None
    ) -> None: ...

    async def recv(self, timeout: float = 2.0) -> Any: ...

    async def close(self) -> None: ...


@dataclass
class FaultInjector:
    active: list[tuple[str, dict[str, Any]]] = field(default_factory=list)

    def add(self, name: str, **options: Any) -> None:
        """Queue a generic fault request for the next device send."""
        self.active.append((name, options))

    def take(self) -> dict[str, Any] | None:
        # Consume once so a test cannot accidentally carry a fault into its next send.
        if not self.active:
            return None
        name, options = self.active.pop(0)
        return {"fault": name, **options}

    def clear(self) -> None:
        self.active.clear()


class SimulatedDevice:
    def __init__(self, bus: Bus, fault: FaultInjector) -> None:
        self.bus = bus
        self.fault = fault

    async def send(self, identifier: int, source: int, payload: bytes) -> None:
        await self.bus.send(identifier, source, payload, fault=self.fault.take())

    async def recv(self, timeout: float = 2.0) -> Any:
        return await self.bus.recv(timeout)


@pytest.fixture
async def device(bus: Bus, fault_injector: FaultInjector):
    """Yield a generic device over the separately provided ``bus`` fixture."""
    yield SimulatedDevice(bus, fault_injector)


@pytest.fixture
def fault_injector() -> FaultInjector:
    return FaultInjector()
