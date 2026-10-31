use j1939_core::{Frame, TickMillis, GLOBAL_ADDRESS, PGN_TP_CM, PGN_TP_DT};
use j1939_tp::{BamTransmitter, Reassembler, TpError};

fn start(receiver: &mut Reassembler) {
    let mut transmitter = BamTransmitter::new(0xEF00, 33, &[1]).unwrap();
    let cm = transmitter.poll(TickMillis(0)).unwrap();
    receiver.on_frame(&cm, TickMillis(0)).unwrap();
}

fn assert_new_session_starts(receiver: &mut Reassembler) {
    assert_eq!(receiver.active_sessions(), 0);
    start(receiver);
    assert_eq!(receiver.active_sessions(), 1);
}

#[test]
fn poll_before_inter_packet_interval_returns_none() {
    let mut transmitter = BamTransmitter::new(0xEF00, 33, &[1]).unwrap();
    assert_eq!(transmitter.poll(TickMillis(0)).unwrap().pgn(), PGN_TP_CM);
    assert_eq!(transmitter.poll(TickMillis(49)), None);
    assert_eq!(transmitter.poll(TickMillis(50)).unwrap().pgn(), PGN_TP_DT);
}

#[test]
fn poll_after_done_returns_none() {
    let mut transmitter = BamTransmitter::new(0xEF00, 33, &[1]).unwrap();
    assert!(transmitter.poll(TickMillis(0)).is_some());
    assert!(transmitter.poll(TickMillis(50)).is_some());
    assert!(transmitter.is_done());
    assert_eq!(transmitter.poll(TickMillis(100)), None);
}

#[test]
fn reassembler_accepts_new_session_after_complete() {
    let mut receiver = Reassembler::new();
    let mut transmitter = BamTransmitter::new(0xEF00, 33, &[7]).unwrap();
    let cm = transmitter.poll(TickMillis(0)).unwrap();
    receiver.on_frame(&cm, TickMillis(0)).unwrap();
    let dt = transmitter.poll(TickMillis(50)).unwrap();
    assert_eq!(
        receiver
            .on_frame(&dt, TickMillis(50))
            .unwrap()
            .unwrap()
            .payload,
        [7]
    );
    assert_new_session_starts(&mut receiver);
}

#[test]
fn reassembler_accepts_new_session_after_timeout() {
    let mut receiver = Reassembler::new();
    start(&mut receiver);
    assert_eq!(receiver.expire_idle(TickMillis(750)).len(), 1);
    assert_new_session_starts(&mut receiver);
}

#[test]
fn reassembler_accepts_new_session_after_abort() {
    let mut receiver = Reassembler::new();
    start(&mut receiver);
    let abort = Frame::new(PGN_TP_CM, 33, GLOBAL_ADDRESS, [0xFF, 3, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        receiver.on_frame(&abort, TickMillis(10)),
        Err(TpError::Aborted(3))
    );
    assert_new_session_starts(&mut receiver);
}

#[test]
fn reassembler_accepts_new_session_after_sequence_mismatch() {
    let mut receiver = Reassembler::new();
    let cm = Frame::new(
        PGN_TP_CM,
        33,
        GLOBAL_ADDRESS,
        [0x20, 8, 0, 2, 0xFF, 0, 0xEF, 0],
    );
    receiver.on_frame(&cm, TickMillis(0)).unwrap();
    let out_of_order = Frame::new(PGN_TP_DT, 33, GLOBAL_ADDRESS, [2, 0, 0, 0, 0, 0, 0, 0]);
    assert!(matches!(
        receiver.on_frame(&out_of_order, TickMillis(1)),
        Err(TpError::SequenceMismatch { .. })
    ));
    assert_new_session_starts(&mut receiver);
}
