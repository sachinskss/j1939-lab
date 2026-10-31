# Test coverage matrix

This matrix lists the named edge tests requested for the implementation. The
six fault scenarios remain in `crates/j1939-tp/tests/fault.rs`.

| Class | Test | Layer | Result |
|---|---|---|---|
| Boundary | `payload_zero_bytes_is_rejected_explicitly` | L1 | PASS locally |
| Boundary | `payload_one_byte_roundtrips` | L1 | PASS locally |
| Boundary | `payload_seven_bytes_roundtrips_in_one_dt_frame` | L1 | PASS locally |
| Boundary | `payload_eight_bytes_roundtrips_in_two_dt_frames` | L1 | PASS locally |
| Boundary | `payload_1785_bytes_roundtrips_in_255_dt_frames` | L1 | PASS locally |
| Boundary | `payload_1786_bytes_is_rejected` | L1 | PASS locally |
| Boundary | `timeout_at_749_milliseconds_does_not_expire` | L1 | PASS locally |
| Boundary | `timeout_at_750_milliseconds_expires` | L1 | PASS locally |
| Boundary | `timeout_at_751_milliseconds_expires` | L1 | PASS locally |
| State | `poll_before_inter_packet_interval_returns_none` | L1 | PASS locally |
| State | `poll_after_done_returns_none` | L1 | PASS locally |
| State | `reassembler_accepts_new_session_after_complete` | L1 | PASS locally |
| State | `reassembler_accepts_new_session_after_timeout` | L1 | PASS locally |
| State | `reassembler_accepts_new_session_after_abort` | L1 | PASS locally |
| State | `reassembler_accepts_new_session_after_sequence_mismatch` | L1 | PASS locally |
| Frame | `frame_default_priority_is_seven` | L1 | PASS locally |
| Frame | `frame_custom_priority_is_preserved` | L1 | PASS locally |
| Frame | `frame_rejects_priority_outside_three_bits` | L1 | PASS locally |
| Frame | `frame_truncates_data_over_eight_bytes` | L1 | PASS locally |
| Frame | `frame_pads_data_under_eight_bytes` | L1 | PASS locally |
| Frame | `pgn_zero_roundtrips` | L1 | PASS locally |
| Frame | `maximum_eighteen_bit_pgn_roundtrips` | L1 | PASS locally |
| Frame | `extended_data_page_bit_roundtrips` | L1 | PASS locally |
| Frame | `pdu2_destination_is_broadcast_even_when_argument_differs` | L1 | PASS locally |
| Frame | `pdu1_preserves_broadcast_null_and_specific_destinations` | L1 | PASS locally |
| Bridge | `sdk_handles_malformed_json_from_cli` | L3 | Pending |
| Bridge | `sdk_handles_cli_crash` | L3 | Pending |
| Bridge | `sdk_respects_wall_clock_timeout` | L3 | Pending |
| Bridge | `sdk_closes_subprocess_on_success` | L3 | Pending |

RTS-to-session is the sole unimplemented L1 receive transition. RTS/CTS remains
out of scope; see [design-notes.md](design-notes.md).
