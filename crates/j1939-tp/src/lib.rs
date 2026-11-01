#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{collections::BTreeMap, vec::Vec};
use core::fmt;
use j1939_core::{Frame, TickMillis, GLOBAL_ADDRESS, PGN_TP_CM, PGN_TP_DT};

const MAX_BAM_PAYLOAD: usize = 1785;
const BAM_TIMEOUT_MS: u64 = 750;
const PACKET_GAP_MS: u64 = 50;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub pgn: u32,
    pub source: u8,
    pub destination: u8,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TpError {
    PayloadTooLarge(usize),
    SequenceMismatch { got: u8, expected: u8 },
    Aborted(u8),
    Timeout { source: u8, pgn: u32 },
    MalformedCm,
}

impl fmt::Display for TpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge(0) => f.write_str("empty payload is not a valid BAM message"),
            Self::PayloadTooLarge(n) => write!(f, "payload size {n} exceeds 1785 bytes"),
            Self::SequenceMismatch { got, expected } => {
                write!(f, "packet sequence {got}, expected {expected}")
            }
            Self::Aborted(code) => write!(f, "transport aborted with reason {code}"),
            Self::Timeout { source, pgn } => write!(
                f,
                "transport from source {source} for PGN {pgn:#x} timed out"
            ),
            Self::MalformedCm => f.write_str("malformed TP.CM frame"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ConnKey {
    source: u8,
    destination: u8,
}

#[derive(Debug)]
struct Session {
    pgn: u32,
    expected_size: usize,
    expected_packets: u8,
    next_sequence: u8,
    payload: Vec<u8>,
    last_activity: TickMillis,
}

#[derive(Default, Debug)]
pub struct Reassembler {
    sessions: BTreeMap<ConnKey, Session>,
}

impl Reassembler {
    pub const fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
        }
    }

    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }

    pub fn on_frame(&mut self, frame: &Frame, now: TickMillis) -> Result<Option<Message>, TpError> {
        match frame.pgn() {
            PGN_TP_CM => self.on_cm(frame, now),
            PGN_TP_DT => self.on_dt(frame, now),
            _ => Ok(None),
        }
    }

    pub fn expire_idle(&mut self, now: TickMillis) -> Vec<TpError> {
        let expired: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, s)| now.0.saturating_sub(s.last_activity.0) >= BAM_TIMEOUT_MS)
            .map(|(key, s)| (*key, s.pgn))
            .collect();
        let mut errors = Vec::with_capacity(expired.len());
        for (key, pgn) in expired {
            self.sessions.remove(&key);
            errors.push(TpError::Timeout {
                source: key.source,
                pgn,
            });
        }
        errors
    }

    fn on_cm(&mut self, frame: &Frame, now: TickMillis) -> Result<Option<Message>, TpError> {
        if frame.dlc < 8 {
            return Err(TpError::MalformedCm);
        }
        let d = frame.data;
        let key = ConnKey {
            source: frame.source(),
            destination: GLOBAL_ADDRESS,
        };
        match d[0] {
            0x20 => {
                let size = u16::from_le_bytes([d[1], d[2]]) as usize;
                let packets = d[3];
                let pgn = read_pgn(&d[5..8]);
                // TP.DT carries seven payload bytes per frame. Reject inconsistent
                // counts before allocating a session.
                let expected_packets = size.div_ceil(7);
                if size == 0
                    || size > MAX_BAM_PAYLOAD
                    || packets == 0
                    || packets as usize != expected_packets
                    || d[4] != 0xFF
                {
                    return Err(TpError::MalformedCm);
                }
                self.sessions.insert(
                    key,
                    Session {
                        pgn,
                        expected_size: size,
                        expected_packets: packets,
                        next_sequence: 1,
                        payload: Vec::with_capacity(size),
                        last_activity: now,
                    },
                );
                Ok(None)
            }
            0xFF => {
                self.sessions.remove(&key);
                Err(TpError::Aborted(d[1]))
            }
            _ => Err(TpError::MalformedCm),
        }
    }

    fn on_dt(&mut self, frame: &Frame, now: TickMillis) -> Result<Option<Message>, TpError> {
        if frame.dlc < 8 {
            return Err(TpError::MalformedCm);
        }
        let key = ConnKey {
            source: frame.source(),
            destination: GLOBAL_ADDRESS,
        };
        let Some(session) = self.sessions.get_mut(&key) else {
            return Ok(None);
        };
        let got = frame.data[0];
        if got != session.next_sequence {
            let expected = session.next_sequence;
            self.sessions.remove(&key);
            return Err(TpError::SequenceMismatch { got, expected });
        }
        let remaining = session.expected_size.saturating_sub(session.payload.len());
        let count = remaining.min(7);
        session.payload.extend_from_slice(&frame.data[1..1 + count]);
        session.next_sequence = session.next_sequence.saturating_add(1);
        session.last_activity = now;
        if got == session.expected_packets {
            if session.payload.len() != session.expected_size {
                self.sessions.remove(&key);
                return Err(TpError::MalformedCm);
            }
            let session = self.sessions.remove(&key).expect("session exists");
            return Ok(Some(Message {
                pgn: session.pgn,
                source: key.source,
                destination: GLOBAL_ADDRESS,
                payload: session.payload,
            }));
        }
        if got > session.expected_packets {
            self.sessions.remove(&key);
            return Err(TpError::MalformedCm);
        }
        Ok(None)
    }
}

fn read_pgn(bytes: &[u8]) -> u32 {
    bytes[0] as u32 | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16)
}

#[derive(Debug)]
pub struct BamTransmitter {
    pgn: u32,
    source: u8,
    payload: Vec<u8>,
    packet_count: u8,
    next_sequence: u8,
    last_emit: Option<TickMillis>,
    done: bool,
}

impl BamTransmitter {
    pub fn new(pgn: u32, source: u8, payload: &[u8]) -> Result<Self, TpError> {
        if payload.len() > MAX_BAM_PAYLOAD {
            return Err(TpError::PayloadTooLarge(payload.len()));
        }
        if payload.is_empty() {
            return Err(TpError::PayloadTooLarge(0));
        }
        let packets = payload.len().div_ceil(7) as u8;
        Ok(Self {
            pgn,
            source,
            payload: payload.to_vec(),
            packet_count: packets,
            next_sequence: 0,
            last_emit: None,
            done: false,
        })
    }

    pub fn poll(&mut self, now: TickMillis) -> Option<Frame> {
        if self.done || self.payload.is_empty() {
            self.done = true;
            return None;
        }
        if self
            .last_emit
            .is_some_and(|last| now.0.saturating_sub(last.0) < PACKET_GAP_MS)
        {
            return None;
        }
        let frame = if self.next_sequence == 0 {
            let size = self.payload.len() as u16;
            let p = self.pgn.to_le_bytes();
            self.next_sequence = 1;
            Frame::new(
                PGN_TP_CM,
                self.source,
                GLOBAL_ADDRESS,
                [
                    0x20,
                    size as u8,
                    (size >> 8) as u8,
                    self.packet_count,
                    0xFF,
                    p[0],
                    p[1],
                    p[2],
                ],
            )
        } else {
            let seq = self.next_sequence;
            let start = (seq as usize - 1) * 7;
            let mut data = [0xFF; 8];
            data[0] = seq;
            for (i, byte) in self.payload[start..(start + 7).min(self.payload.len())]
                .iter()
                .enumerate()
            {
                data[i + 1] = *byte;
            }
            if seq == self.packet_count {
                self.done = true;
            } else {
                self.next_sequence += 1;
            }
            Frame::new(PGN_TP_DT, self.source, GLOBAL_ADDRESS, data)
        };
        self.last_emit = Some(now);
        Some(frame)
    }

    pub const fn is_done(&self) -> bool {
        self.done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transmit(payload: &[u8]) -> Result<Message, TpError> {
        let mut tx = BamTransmitter::new(0x00_EF00, 33, payload)?;
        let mut rx = Reassembler::new();
        let mut now = TickMillis(0);
        loop {
            if let Some(frame) = tx.poll(now) {
                if let Some(msg) = rx.on_frame(&frame, now)? {
                    return Ok(msg);
                }
            }
            if tx.is_done() {
                break;
            }
            now.0 += 50;
        }
        Err(TpError::MalformedCm)
    }

    #[test]
    fn bam_roundtrip_reassembles_payload() {
        let payload: Vec<u8> = (0..100).collect();
        let message = transmit(&payload).unwrap();
        assert_eq!(message.payload, payload);
        assert_eq!(message.pgn, 0x00_EF00);
    }

    #[test]
    fn oversized_payload_is_typed() {
        assert_eq!(
            BamTransmitter::new(1, 2, &alloc::vec![0; 2000]).unwrap_err(),
            TpError::PayloadTooLarge(2000)
        );
    }

    #[test]
    fn missing_packet_expires_without_a_clock_read() {
        let mut tx = BamTransmitter::new(1, 33, &[1; 20]).unwrap();
        let mut rx = Reassembler::new();
        let cm = tx.poll(TickMillis(0)).unwrap();
        rx.on_frame(&cm, TickMillis(0)).unwrap();
        assert_eq!(
            rx.expire_idle(TickMillis(750)),
            alloc::vec![TpError::Timeout { source: 33, pgn: 1 }]
        );
    }

    #[test]
    fn out_of_order_packets_return_sequence_mismatch() {
        let mut rx = Reassembler::new();
        let mut tx = BamTransmitter::new(1, 33, &[1; 20]).unwrap();
        let cm = tx.poll(TickMillis(0)).unwrap();
        rx.on_frame(&cm, TickMillis(0)).unwrap();
        let _first = tx.poll(TickMillis(50)).unwrap();
        let second = tx.poll(TickMillis(100)).unwrap();
        assert_eq!(
            rx.on_frame(&second, TickMillis(100)),
            Err(TpError::SequenceMismatch {
                got: 2,
                expected: 1
            })
        );
    }

    #[test]
    fn abort_and_bad_cm_are_typed() {
        let mut rx = Reassembler::new();
        let abort = Frame::new(PGN_TP_CM, 33, GLOBAL_ADDRESS, [0xFF, 3, 0, 0, 0, 1, 0, 0]);
        assert_eq!(rx.on_frame(&abort, TickMillis(0)), Err(TpError::Aborted(3)));
        let malformed = Frame::new(PGN_TP_CM, 33, GLOBAL_ADDRESS, [0x20, 1, 0, 0, 0, 1, 0, 0]);
        assert_eq!(
            rx.on_frame(&malformed, TickMillis(0)),
            Err(TpError::MalformedCm)
        );
    }

    #[test]
    fn duplicate_bam_cm_replaces_the_open_session() {
        let payload: Vec<u8> = (0..100).collect();
        let mut tx = BamTransmitter::new(0xEF00, 33, &payload).unwrap();
        let cm = tx.poll(TickMillis(0)).unwrap();
        let mut rx = Reassembler::new();
        rx.on_frame(&cm, TickMillis(0)).unwrap();
        rx.on_frame(&cm, TickMillis(1)).unwrap();
        let mut complete = None;
        let mut t = 50;
        while !tx.is_done() {
            if let Some(frame) = tx.poll(TickMillis(t)) {
                complete = rx.on_frame(&frame, TickMillis(t)).unwrap().or(complete);
            }
            t += 50;
        }
        assert_eq!(complete.unwrap().payload, payload);
    }
}
