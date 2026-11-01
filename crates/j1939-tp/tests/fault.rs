use j1939_core::{Frame, TickMillis, GLOBAL_ADDRESS, PGN_TP_CM};
use j1939_tp::{BamTransmitter, Reassembler, TpError};

fn cm(size: usize, packets: u8, pgn: u32) -> Frame {
    let p = pgn.to_le_bytes();
    Frame::new(
        PGN_TP_CM,
        33,
        GLOBAL_ADDRESS,
        [
            0x20,
            size as u8,
            (size >> 8) as u8,
            packets,
            0xFF,
            p[0],
            p[1],
            p[2],
        ],
    )
}

#[test]
fn f1_incomplete_session_expires_with_timeout() {
    let mut rx = Reassembler::new();
    rx.on_frame(&cm(20, 3, 0xEF00), TickMillis(0)).unwrap();
    let first = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [1, 1, 2, 3, 4, 5, 6, 7]);
    let second = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [2, 8, 9, 10, 11, 12, 13, 14]);
    rx.on_frame(&first, TickMillis(50)).unwrap();
    rx.on_frame(&second, TickMillis(100)).unwrap();
    assert_eq!(
        rx.expire_idle(TickMillis(850)),
        vec![TpError::Timeout {
            source: 33,
            pgn: 0xEF00
        }]
    );
}

#[test]
fn f1b_dropped_middle_packet_is_reported_when_a_later_packet_arrives() {
    let mut rx = Reassembler::new();
    rx.on_frame(&cm(20, 3, 0xEF00), TickMillis(0)).unwrap();
    let first = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [1, 1, 2, 3, 4, 5, 6, 7]);
    let third = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [3, 15, 16, 17, 18, 19, 20, 21]);
    rx.on_frame(&first, TickMillis(50)).unwrap();
    assert_eq!(
        rx.on_frame(&third, TickMillis(150)),
        Err(TpError::SequenceMismatch {
            got: 3,
            expected: 2
        })
    );
}

#[test]
fn f2_reorder_dt_frames_returns_sequence_mismatch() {
    let mut tx = BamTransmitter::new(0xEF00, 33, &[1; 20]).unwrap();
    let mut rx = Reassembler::new();
    rx.on_frame(&tx.poll(TickMillis(0)).unwrap(), TickMillis(0))
        .unwrap();
    let _packet_one = tx.poll(TickMillis(50)).unwrap();
    let packet_two = tx.poll(TickMillis(100)).unwrap();
    assert_eq!(
        rx.on_frame(&packet_two, TickMillis(100)),
        Err(TpError::SequenceMismatch {
            got: 2,
            expected: 1
        })
    );
}

#[test]
fn f3_abort_mid_stream_returns_reason() {
    let mut rx = Reassembler::new();
    rx.on_frame(&cm(20, 3, 0xEF00), TickMillis(0)).unwrap();
    let first = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [1, 1, 2, 3, 4, 5, 6, 7]);
    rx.on_frame(&first, TickMillis(50)).unwrap();
    let abort = Frame::new(PGN_TP_CM, 33, GLOBAL_ADDRESS, [0xFF, 3, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        rx.on_frame(&abort, TickMillis(100)),
        Err(TpError::Aborted(3))
    );
}

#[test]
fn f4_oversized_payload_is_rejected() {
    assert_eq!(
        BamTransmitter::new(0xEF00, 33, &vec![0; 2000]).unwrap_err(),
        TpError::PayloadTooLarge(2000)
    );
}

#[test]
fn f5_twenty_bytes_in_two_packets_is_malformed() {
    let mut rx = Reassembler::new();
    assert_eq!(
        rx.on_frame(&cm(20, 2, 0xEF00), TickMillis(0)),
        Err(TpError::MalformedCm)
    );
    assert_eq!(rx.active_sessions(), 0);
}

#[test]
fn f5_valid_twenty_byte_announcement_has_three_packets() {
    let mut rx = Reassembler::new();
    assert_eq!(rx.on_frame(&cm(20, 3, 0xEF00), TickMillis(0)), Ok(None));
    assert_eq!(rx.active_sessions(), 1);
}

#[test]
fn f6_duplicate_cm_resets_session_and_drops_first_message() {
    let data = cm(8, 2, 0xEF00);
    let mut rx = Reassembler::new();
    rx.on_frame(&data, TickMillis(0)).unwrap();
    let old_first = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [1, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(rx.on_frame(&old_first, TickMillis(10)).unwrap(), None);
    rx.on_frame(&data, TickMillis(20)).unwrap();
    let new_first = Frame::new(0xEB00, 33, GLOBAL_ADDRESS, [1, 8, 9, 10, 11, 12, 13, 14]);
    assert_eq!(rx.on_frame(&new_first, TickMillis(30)).unwrap(), None);
    let second = Frame::new(
        0xEB00,
        33,
        GLOBAL_ADDRESS,
        [2, 15, 16, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    let message = rx.on_frame(&second, TickMillis(40)).unwrap().unwrap();
    assert_eq!(message.payload, vec![8, 9, 10, 11, 12, 13, 14, 15]);
}
