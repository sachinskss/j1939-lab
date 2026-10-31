use j1939_core::{Frame, TickMillis, PGN_TP_CM, PGN_TP_DT};
use j1939_tp::{BamTransmitter, Message, Reassembler, TpError};

fn roundtrip(payload: &[u8]) -> Result<(Message, Vec<Frame>), TpError> {
    let mut tx = BamTransmitter::new(0xEF00, 33, payload)?;
    let mut rx = Reassembler::new();
    let mut frames = Vec::new();
    let mut now = TickMillis(0);
    let mut message = None;
    while !tx.is_done() {
        if let Some(frame) = tx.poll(now) {
            message = rx.on_frame(&frame, now)?.or(message);
            frames.push(frame);
        }
        now.0 += 50;
    }
    Ok((message.ok_or(TpError::MalformedCm)?, frames))
}

fn data_packet_count(frames: &[Frame]) -> usize {
    frames
        .iter()
        .filter(|frame| frame.pgn() == PGN_TP_DT)
        .count()
}

#[test]
fn payload_zero_bytes_is_rejected_explicitly() {
    assert_eq!(
        BamTransmitter::new(0xEF00, 33, &[]).unwrap_err(),
        TpError::PayloadTooLarge(0)
    );
}

#[test]
fn payload_one_byte_roundtrips() {
    let (message, frames) = roundtrip(&[0]).unwrap();
    assert_eq!(message.payload, [0]);
    assert_eq!(data_packet_count(&frames), 1);
}

#[test]
fn payload_seven_bytes_roundtrips_in_one_dt_frame() {
    let payload = [0xA5; 7];
    let (message, frames) = roundtrip(&payload).unwrap();
    assert_eq!(message.payload.len(), 7);
    assert_eq!(data_packet_count(&frames), 1);
}

#[test]
fn payload_eight_bytes_roundtrips_in_two_dt_frames() {
    let payload = [0xA5; 8];
    let (message, frames) = roundtrip(&payload).unwrap();
    assert_eq!(message.payload.len(), 8);
    assert_eq!(data_packet_count(&frames), 2);
}

#[test]
fn payload_1785_bytes_roundtrips_in_255_dt_frames() {
    let payload = vec![0x5A; 1785];
    let (message, frames) = roundtrip(&payload).unwrap();
    assert_eq!(message.payload, payload);
    assert_eq!(data_packet_count(&frames), 255);
    let cm = frames
        .iter()
        .find(|frame| frame.pgn() == PGN_TP_CM)
        .unwrap();
    assert_eq!(cm.data[3], 255);
}

#[test]
fn payload_1786_bytes_is_rejected() {
    assert_eq!(
        BamTransmitter::new(0xEF00, 33, &vec![0; 1786]).unwrap_err(),
        TpError::PayloadTooLarge(1786)
    );
}

fn start_session(receiver: &mut Reassembler, now: TickMillis) {
    let mut transmitter = BamTransmitter::new(0xEF00, 33, &[1]).unwrap();
    let cm = transmitter.poll(now).unwrap();
    assert_eq!(cm.pgn(), PGN_TP_CM);
    assert_eq!(receiver.on_frame(&cm, now), Ok(None));
}

#[test]
fn timeout_at_749_milliseconds_does_not_expire() {
    let mut receiver = Reassembler::new();
    start_session(&mut receiver, TickMillis(0));
    assert!(receiver.expire_idle(TickMillis(749)).is_empty());
    assert_eq!(receiver.active_sessions(), 1);
}

#[test]
fn timeout_at_750_milliseconds_expires() {
    let mut receiver = Reassembler::new();
    start_session(&mut receiver, TickMillis(0));
    assert_eq!(
        receiver.expire_idle(TickMillis(750)),
        vec![TpError::Timeout {
            source: 33,
            pgn: 0xEF00
        }]
    );
    assert_eq!(receiver.active_sessions(), 0);
}

#[test]
fn timeout_at_751_milliseconds_expires() {
    let mut receiver = Reassembler::new();
    start_session(&mut receiver, TickMillis(0));
    assert_eq!(receiver.expire_idle(TickMillis(751)).len(), 1);
    assert_eq!(receiver.active_sessions(), 0);
}
