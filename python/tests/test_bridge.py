import asyncio

import pytest

from j1939_sdk import Bus, InternalError


class FakeProcess:
    def __init__(self, returncode=0):
        self.stdout = asyncio.StreamReader()
        self.stderr = asyncio.StreamReader()
        self.stdin = None
        self.returncode = returncode


@pytest.mark.asyncio
async def test_sdk_handles_malformed_json_from_cli():
    process = FakeProcess()
    process.stdout.feed_data(b"not json\n")
    bus = Bus(process)
    with pytest.raises(InternalError) as error:
        await bus._read_response()
    assert error.value.variant == "InternalError"


@pytest.mark.asyncio
async def test_sdk_handles_cli_crash():
    process = FakeProcess(returncode=101)
    process.stdout.feed_eof()
    process.stderr.feed_data(b"panic in subprocess")
    process.stderr.feed_eof()
    bus = Bus(process)
    with pytest.raises(InternalError) as error:
        await bus._read_response()
    assert error.value.variant == "InternalError"
    assert "panic in subprocess" in str(error.value)


@pytest.mark.asyncio
async def test_sdk_respects_wall_clock_timeout():
    bus = Bus(FakeProcess())
    with pytest.raises(asyncio.TimeoutError):
        await bus.recv(timeout=0.001)


@pytest.mark.asyncio
async def test_sdk_closes_subprocess_on_success(sdk_bus):
    process = sdk_bus._process
    await sdk_bus.close()
    assert process.returncode is not None
