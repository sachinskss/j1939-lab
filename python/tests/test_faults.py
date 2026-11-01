"""SDK-level integration tests prove injected requests alter transport outcomes."""

import pytest

from j1939_sdk import Aborted, MalformedCm, PayloadTooLarge, SequenceMismatch, Timeout


@pytest.mark.asyncio
async def test_f1_missing_final_packet_expires_at_sdk(sdk_bus):
    await sdk_bus.send_bam(0xEF00, 33, bytes(range(20)), fault={"fault": "drop", "packet": 3})
    with pytest.raises(Timeout):
        await sdk_bus.recv()


@pytest.mark.asyncio
async def test_f2_reordered_packets_return_sequence_error_at_sdk(sdk_bus):
    await sdk_bus.send_bam(0xEF00, 33, bytes(range(20)), fault={"fault": "reorder"})
    with pytest.raises(SequenceMismatch):
        await sdk_bus.recv()


@pytest.mark.asyncio
async def test_f3_midstream_abort_preserves_reason_at_sdk(sdk_bus):
    await sdk_bus.send_bam(0xEF00, 33, bytes(range(20)), fault={"fault": "abort", "after_packets": 1, "reason": 3})
    with pytest.raises(Aborted, match="reason 3"):
        await sdk_bus.recv()


@pytest.mark.asyncio
async def test_f4_oversized_payload_is_typed_at_sdk(sdk_bus):
    await sdk_bus.send_bam(0xEF00, 33, bytes(2000))
    with pytest.raises(PayloadTooLarge):
        await sdk_bus.recv()


@pytest.mark.asyncio
async def test_f5_invalid_announcement_is_typed_at_sdk(sdk_bus):
    await sdk_bus.send_bam(0xEF00, 33, bytes(range(20)), fault={"fault": "malformed"})
    with pytest.raises(MalformedCm):
        await sdk_bus.recv()


@pytest.mark.asyncio
async def test_f6_duplicate_announcement_resets_at_sdk(sdk_bus):
    payload = bytes(range(100))
    await sdk_bus.send_bam(0xEF00, 33, payload, fault={"fault": "duplicate"})
    assert (await sdk_bus.recv()).payload == payload
