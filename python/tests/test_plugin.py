import pytest


@pytest.mark.asyncio
async def test_device_fixture_uses_generic_bus(device):
    payload = bytes(range(32))
    await device.send(0xEF00, 33, payload)
    message = await device.recv()
    assert message.payload == payload


@pytest.mark.asyncio
async def test_fault_fixture_changes_device_result(device, fault_injector):
    fault_injector.add("reorder", first_packet=1, second_packet=2)
    await device.send(0xEF00, 33, bytes(range(20)))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "SequenceMismatch"
    assert fault_injector.active == []
