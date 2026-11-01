import pytest
from j1939_sdk import SequenceMismatch


@pytest.mark.asyncio
async def test_device_fixture_uses_generic_bus(device):
    payload = bytes(range(32))
    await device.send(0xEF00, 33, payload)
    message = await device.recv()
    assert message.payload == payload


@pytest.mark.asyncio
async def test_fault_injector_forwards_fault_through_adapter(device, fault_injector):
    fault_injector.add("reorder", first_packet=1, second_packet=2)
    await device.send(0xEF00, 33, bytes(range(20)))
    # Assert the adapter returns the transport's typed failure, not just a recorded request.
    with pytest.raises(SequenceMismatch):
        await device.recv()
    assert fault_injector.active == []
