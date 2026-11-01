use j1939_core::TickMillis;
use j1939_sim::{FaultBus, FaultPlan, VirtualBus};
use j1939_tp::{BamTransmitter, Message, Reassembler, TpError};

/// Deliver each simulated transmitter frame through the configured bus and the
/// same reassembler used by the shell, returning its state when transmission ends.
fn transmit(
    payload: &[u8],
    plan: FaultPlan,
) -> Result<(Option<Message>, Reassembler, TickMillis), TpError> {
    let mut transmitter = BamTransmitter::new(0xEF00, 33, payload)?;
    let bus = VirtualBus::new();
    let receiver = bus.subscribe();
    let mut fault_bus = FaultBus::new(bus, plan);
    let mut reassembler = Reassembler::new();
    let mut now = TickMillis(0);

    while !transmitter.is_done() {
        if let Some(frame) = transmitter.poll(now) {
            fault_bus.broadcast(frame);
            while let Ok(received) = receiver.try_recv() {
                if let Some(message) = reassembler.on_frame(&received, now)? {
                    return Ok((Some(message), reassembler, now));
                }
            }
        }
        now.0 += 50;
    }
    Ok((None, reassembler, now))
}

#[test]
fn f1_fault_bus_drops_final_packet_and_reassembler_times_out() {
    let (message, mut reassembler, now) =
        transmit(&(0..20).collect::<Vec<u8>>(), FaultPlan::DropPacket(3)).unwrap();
    assert!(message.is_none());
    assert_eq!(reassembler.active_sessions(), 1);
    assert_eq!(
        reassembler.expire_idle(TickMillis(now.0 + 750)),
        vec![TpError::Timeout {
            source: 33,
            pgn: 0xEF00,
        }]
    );
}

#[test]
fn f2_fault_bus_reorders_frames_and_reassembler_rejects_sequence() {
    assert_eq!(
        transmit(
            &(0..20).collect::<Vec<u8>>(),
            FaultPlan::ReorderPackets(1, 2)
        )
        .unwrap_err(),
        TpError::SequenceMismatch {
            got: 2,
            expected: 1,
        }
    );
}

#[test]
fn f3_fault_bus_abort_is_typed_by_reassembler() {
    assert_eq!(
        transmit(
            &(0..20).collect::<Vec<u8>>(),
            FaultPlan::AbortAfter {
                packets: 1,
                reason: 3,
            },
        )
        .unwrap_err(),
        TpError::Aborted(3)
    );
}

#[test]
fn f4_oversized_payload_is_rejected_before_any_bus_frame() {
    let bus = VirtualBus::new();
    let receiver = bus.subscribe();
    let reassembler = Reassembler::new();
    assert_eq!(
        BamTransmitter::new(0xEF00, 33, &vec![0; 2000]).unwrap_err(),
        TpError::PayloadTooLarge(2000)
    );
    assert!(receiver.try_recv().is_err());
    assert_eq!(reassembler.active_sessions(), 0);
}

#[test]
fn f5_inconsistent_announcement_is_rejected_by_reassembler() {
    assert_eq!(
        transmit(
            &(0..20).collect::<Vec<u8>>(),
            FaultPlan::MalformedAnnouncement
        )
        .unwrap_err(),
        TpError::MalformedCm
    );
}

#[test]
fn f6_duplicate_announcement_resets_session_and_completes_new_message() {
    let payload: Vec<u8> = (0..100).collect();
    let (message, _, _) = transmit(&payload, FaultPlan::DuplicateAnnouncement).unwrap();
    assert_eq!(message.unwrap().payload, payload);
}
