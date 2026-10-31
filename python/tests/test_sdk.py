import pytest
from j1939_sdk import PayloadTooLarge


@pytest.mark.asyncio
async def test_async_bridge_roundtrip(bus):
    payload = bytes(range(100))
    await bus.send_bam(0xEF00, 33, payload)
    message = await bus.recv()
    assert message.pgn == 0xEF00
    assert message.source == 33
    assert message.payload == payload


@pytest.mark.asyncio
async def test_bridge_preserves_typed_payload_error(bus):
    await bus.send_bam(0xEF00, 33, bytes(2000))
    with pytest.raises(PayloadTooLarge):
        await bus.recv()
