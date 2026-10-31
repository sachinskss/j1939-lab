#![forbid(unsafe_code)]

use j1939_core::Frame;
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
}
