# Test coverage matrix

This matrix lists named edge and fault tests. Scenario results come from the
generated `verification-evidence` CI artifact, not from this inventory.

The local suite contains 77 test functions: 54 Rust and 23 Python. The six
fault cases run at L1 (core), L2 (virtual bus), L3 (SDK), and L4 (pytest plugin).
F1 names the no-further-frames timeout. The separate F1b test skips a middle
packet and confirms that the next packet returns `SequenceMismatch`.
F5 rejects 20 bytes announced as 2 packets because 3 are required; the paired
valid case announces 3 packets and opens a session.

| Scenario | L1 core case | L2 bus case | L3 SDK case | L4 plugin case |
|---|---|---|---|---|
| F1 | `f1_incomplete_transmission_expires_with_timeout` | `f1_fault_bus_drops_final_packet_and_reassembler_times_out` | `test_f1_missing_final_packet_expires_at_sdk` | `test_f1_missing_final_packet_expires_at_device` |
| F2 | `f2_reorder_dt_frames_returns_sequence_mismatch` | `f2_fault_bus_reorders_frames_and_reassembler_rejects_sequence` | `test_f2_reordered_packets_return_sequence_error_at_sdk` | `test_f2_reordered_packets_return_sequence_error_at_device` |
| F3 | `f3_abort_mid_stream_returns_reason` | `f3_fault_bus_abort_is_typed_by_reassembler` | `test_f3_midstream_abort_preserves_reason_at_sdk` | `test_f3_midstream_abort_preserves_reason_at_device` |
| F4 | `f4_oversized_payload_is_rejected` | `f4_oversized_payload_is_rejected_before_any_bus_frame` | `test_f4_oversized_payload_is_typed_at_sdk` | `test_f4_oversized_payload_is_typed_at_device` |
| F5 | `f5_twenty_bytes_in_two_packets_is_malformed` | `f5_inconsistent_announcement_is_rejected_by_reassembler` | `test_f5_invalid_announcement_is_typed_at_sdk` | `test_f5_invalid_announcement_is_typed_at_device` |
| F6 | `f6_duplicate_cm_resets_session_and_drops_first_message` | `f6_duplicate_announcement_resets_session_and_completes_new_message` | `test_f6_duplicate_announcement_resets_at_sdk` | `test_f6_duplicate_announcement_resets_at_device` |

| Class | Test | Layer | Inventory |
|---|---|---|---|
| Boundary | `payload_zero_bytes_is_rejected_explicitly` | L1 | Tracked |
| Boundary | `payload_one_byte_roundtrips` | L1 | Tracked |
| Boundary | `payload_seven_bytes_roundtrips_in_one_dt_frame` | L1 | Tracked |
| Boundary | `payload_eight_bytes_roundtrips_in_two_dt_frames` | L1 | Tracked |
| Boundary | `payload_1785_bytes_roundtrips_in_255_dt_frames` | L1 | Tracked |
| Boundary | `payload_1786_bytes_is_rejected` | L1 | Tracked |
| Boundary | `timeout_at_749_milliseconds_does_not_expire` | L1 | Tracked |
| Boundary | `timeout_at_750_milliseconds_expires` | L1 | Tracked |
| Boundary | `timeout_at_751_milliseconds_expires` | L1 | Tracked |
| State | `poll_before_inter_packet_interval_returns_none` | L1 | Tracked |
| State | `poll_after_done_returns_none` | L1 | Tracked |
| State | `reassembler_accepts_new_session_after_complete` | L1 | Tracked |
| State | `reassembler_accepts_new_session_after_timeout` | L1 | Tracked |
| State | `reassembler_accepts_new_session_after_abort` | L1 | Tracked |
| State | `reassembler_accepts_new_session_after_sequence_mismatch` | L1 | Tracked |
| Frame | `frame_default_priority_is_seven` | L1 | Tracked |
| Frame | `frame_custom_priority_is_preserved` | L1 | Tracked |
| Frame | `frame_rejects_priority_outside_three_bits` | L1 | Tracked |
| Frame | `frame_truncates_data_over_eight_bytes` | L1 | Tracked |
| Frame | `frame_pads_data_under_eight_bytes` | L1 | Tracked |
| Frame | `pgn_zero_roundtrips` | L1 | Tracked |
| Frame | `maximum_eighteen_bit_pgn_roundtrips` | L1 | Tracked |
| Frame | `extended_data_page_bit_roundtrips` | L1 | Tracked |
| Frame | `pdu2_destination_is_broadcast_even_when_argument_differs` | L1 | Tracked |
| Frame | `pdu1_preserves_broadcast_null_and_specific_destinations` | L1 | Tracked |
| Bridge | `sdk_handles_malformed_json_from_cli` | L3 | Tracked |
| Bridge | `sdk_handles_cli_crash` | L3 | Tracked |
| Bridge | `sdk_respects_wall_clock_timeout` | L3 | Tracked |
| Bridge | `sdk_closes_subprocess_on_success` | L3 | Tracked |

RTS-to-session is the sole unimplemented L1 receive transition. RTS/CTS remains
out of scope; see [design-notes.md](design-notes.md).
