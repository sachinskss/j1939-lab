#![forbid(unsafe_code)]

use j1939_core::{Frame, GLOBAL_ADDRESS, PGN_TP_CM, PGN_TP_DT};
use std::sync::{mpsc, Arc, Mutex};

#[derive(Clone, Default)]
pub struct VirtualBus {
    senders: Arc<Mutex<Vec<mpsc::Sender<Frame>>>>,
}

impl VirtualBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> mpsc::Receiver<Frame> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut senders) = self.senders.lock() {
            senders.push(sender);
        }
        receiver
    }

    pub fn broadcast(&self, frame: Frame) {
        if let Ok(mut senders) = self.senders.lock() {
            senders.retain(|sender| sender.send(frame).is_ok());
        }
    }
}

/// Deterministic faults applied to outgoing frames on the simulation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultPlan {
    DropPacket(u8),
    ReorderPackets(u8, u8),
    AbortAfter { packets: u8, reason: u8 },
    MalformedAnnouncement,
    DuplicateAnnouncement,
}

/// A virtual bus decorator that mutates only the configured transport frames.
///
/// Keeping this behavior at the bus boundary lets the transmitter and
/// reassembler run unchanged, so integration tests observe real protocol errors.
pub struct FaultBus {
    bus: VirtualBus,
    plan: FaultPlan,
    emitted_packets: u8,
    held_frame: Option<Frame>,
    fault_applied: bool,
}

impl FaultBus {
    pub fn new(bus: VirtualBus, plan: FaultPlan) -> Self {
        Self {
            bus,
            plan,
            emitted_packets: 0,
            held_frame: None,
            fault_applied: false,
        }
    }

    pub fn broadcast(&mut self, mut frame: Frame) {
        if self.fault_applied {
            self.bus.broadcast(frame);
            return;
        }

        if frame.pgn() == PGN_TP_CM {
            match self.plan {
                FaultPlan::MalformedAnnouncement if frame.data[0] == 0x20 => {
                    // Preserve the announced byte size but make its packet count inconsistent.
                    frame.data[3] = frame.data[3].saturating_sub(1);
                    self.fault_applied = true;
                }
                FaultPlan::DuplicateAnnouncement if frame.data[0] == 0x20 => {
                    self.bus.broadcast(frame);
                    self.bus.broadcast(frame);
                    self.fault_applied = true;
                    return;
                }
                _ => {}
            }
            self.bus.broadcast(frame);
            return;
        }

        if frame.pgn() != PGN_TP_DT {
            self.bus.broadcast(frame);
            return;
        }

        let sequence = frame.data[0];
        self.emitted_packets = self.emitted_packets.saturating_add(1);
        match self.plan {
            FaultPlan::DropPacket(target) if sequence == target => {
                // A missing final packet becomes an idle timeout; a missing middle
                // packet is exposed when the next sequence number arrives.
                self.fault_applied = true;
            }
            FaultPlan::ReorderPackets(first, _) if sequence == first => {
                self.held_frame = Some(frame);
            }
            FaultPlan::ReorderPackets(_, second) if sequence == second => {
                self.bus.broadcast(frame);
                if let Some(held) = self.held_frame.take() {
                    self.bus.broadcast(held);
                }
                self.fault_applied = true;
            }
            FaultPlan::AbortAfter { packets, reason } if self.emitted_packets > packets => {
                let abort = Frame::new(
                    PGN_TP_CM,
                    frame.source(),
                    GLOBAL_ADDRESS,
                    [0xFF, reason, 0, 0, 0, 0, 0, 0],
                );
                self.bus.broadcast(abort);
                self.fault_applied = true;
            }
            _ => self.bus.broadcast(frame),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcasts_to_each_subscriber() {
        let bus = VirtualBus::new();
        let a = bus.subscribe();
        let b = bus.subscribe();
        let frame = Frame::new(0xEF00, 3, 0xFF, [7; 8]);
        bus.broadcast(frame);
        assert_eq!(a.try_recv(), Ok(frame));
        assert_eq!(b.try_recv(), Ok(frame));
    }

    fn sample_frames() -> [Frame; 4] {
        [
            Frame::new(
                PGN_TP_CM,
                33,
                GLOBAL_ADDRESS,
                [0x20, 20, 0, 3, 0xFF, 0, 0xEF, 0],
            ),
            Frame::new(PGN_TP_DT, 33, GLOBAL_ADDRESS, [1, 1, 2, 3, 4, 5, 6, 7]),
            Frame::new(PGN_TP_DT, 33, GLOBAL_ADDRESS, [2, 8, 9, 10, 11, 12, 13, 14]),
            Frame::new(
                PGN_TP_DT,
                33,
                GLOBAL_ADDRESS,
                [3, 15, 16, 17, 18, 19, 20, 21],
            ),
        ]
    }

    #[test]
    fn f1_fault_bus_drops_selected_packet() {
        let bus = VirtualBus::new();
        let rx = bus.subscribe();
        let mut fault_bus = FaultBus::new(bus, FaultPlan::DropPacket(3));
        for frame in sample_frames() {
            fault_bus.broadcast(frame);
        }
        let frames: Vec<_> = rx.try_iter().collect();
        assert_eq!(
            frames.iter().map(|f| f.data[0]).collect::<Vec<_>>(),
            [0x20, 1, 2]
        );
    }

    #[test]
    fn f2_fault_bus_reorders_selected_packets() {
        let bus = VirtualBus::new();
        let rx = bus.subscribe();
        let mut fault_bus = FaultBus::new(bus, FaultPlan::ReorderPackets(1, 2));
        for frame in sample_frames() {
            fault_bus.broadcast(frame);
        }
        let frames: Vec<_> = rx.try_iter().collect();
        assert_eq!(
            frames.iter().map(|f| f.data[0]).collect::<Vec<_>>(),
            [0x20, 2, 1, 3]
        );
    }

    #[test]
    fn f3_fault_bus_replaces_next_packet_with_abort() {
        let bus = VirtualBus::new();
        let rx = bus.subscribe();
        let mut fault_bus = FaultBus::new(
            bus,
            FaultPlan::AbortAfter {
                packets: 1,
                reason: 3,
            },
        );
        for frame in sample_frames() {
            fault_bus.broadcast(frame);
        }
        let frames: Vec<_> = rx.try_iter().collect();
        assert!(frames
            .iter()
            .any(|frame| frame.pgn() == PGN_TP_CM && frame.data[0..2] == [0xFF, 3]));
    }

    #[test]
    fn f5_fault_bus_corrupts_announcement_packet_count() {
        let bus = VirtualBus::new();
        let rx = bus.subscribe();
        let mut fault_bus = FaultBus::new(bus, FaultPlan::MalformedAnnouncement);
        fault_bus.broadcast(sample_frames()[0]);
        assert_eq!(rx.try_recv().unwrap().data[3], 2);
    }

    #[test]
    fn f6_fault_bus_broadcasts_duplicate_announcement() {
        let bus = VirtualBus::new();
        let rx = bus.subscribe();
        let mut fault_bus = FaultBus::new(bus, FaultPlan::DuplicateAnnouncement);
        fault_bus.broadcast(sample_frames()[0]);
        assert_eq!(rx.try_iter().count(), 2);
    }
}
