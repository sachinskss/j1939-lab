"""L4 cases exercise generic fixtures with the SDK adapter installed."""

import pytest


@pytest.mark.asyncio
async def test_f1_missing_final_packet_expires_at_device(device, fault_injector):
    fault_injector.add("drop", packet=3)
    await device.send(0xEF00, 33, bytes(range(20)))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "Timeout"


@pytest.mark.asyncio
async def test_f2_reordered_packets_return_sequence_error_at_device(device, fault_injector):
    fault_injector.add("reorder")
    await device.send(0xEF00, 33, bytes(range(20)))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "SequenceMismatch"


@pytest.mark.asyncio
async def test_f3_midstream_abort_preserves_reason_at_device(device, fault_injector):
    fault_injector.add("abort", after_packets=1, reason=3)
    await device.send(0xEF00, 33, bytes(range(20)))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "Aborted"
    assert "reason 3" in str(result.value)


@pytest.mark.asyncio
async def test_f4_oversized_payload_is_typed_at_device(device):
    await device.send(0xEF00, 33, bytes(2000))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "PayloadTooLarge"


@pytest.mark.asyncio
async def test_f5_invalid_announcement_is_typed_at_device(device, fault_injector):
    fault_injector.add("malformed")
    await device.send(0xEF00, 33, bytes(range(20)))
    with pytest.raises(Exception) as result:
        await device.recv()
    assert result.value.__class__.__name__ == "MalformedCm"


@pytest.mark.asyncio
async def test_f6_duplicate_announcement_resets_at_device(device, fault_injector):
    payload = bytes(range(100))
    fault_injector.add("duplicate")
    await device.send(0xEF00, 33, payload)
    assert (await device.recv()).payload == payload
