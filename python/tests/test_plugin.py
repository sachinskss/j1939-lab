import pytest


@pytest.mark.asyncio
async def test_device_fixture_uses_generic_bus(device):
    payload = bytes(range(32))
    await device.send(0xEF00, 33, payload)
    message = await device.recv()
    assert message.payload == payload


def test_fault_fixture_is_shared_with_device(device, fault_injector):
    fault_injector.add("drop", packet=2)
    assert device.fault is fault_injector
    assert fault_injector.active == [("drop", {"packet": 2})]
