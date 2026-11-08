//! Core J1939 frame, identifier, and NAME types with no I/O dependencies.
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

/// PGN for J1939 transport-protocol data transfer frames.
pub const PGN_TP_DT: u32 = 0x00_EB00;
/// PGN for J1939 transport-protocol connection-management frames.
pub const PGN_TP_CM: u32 = 0x00_EC00;
/// PGN for address-claimed messages.
pub const PGN_ADDRESS_CLAIMED: u32 = 0x00_EE00;
/// PGN for request messages.
pub const PGN_REQUEST: u32 = 0x00_EA00;
/// Global destination address used for broadcast messages.
pub const GLOBAL_ADDRESS: u8 = 0xFF;

/// Errors returned when constructing or modifying a CAN frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    /// The requested priority is outside the valid three-bit range.
    InvalidPriority(u8),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPriority(priority) => write!(f, "priority {priority} is outside 0..=7"),
        }
    }
}

/// An eight-byte J1939 CAN frame and its 29-bit identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame {
    /// Encoded extended CAN identifier.
    pub id: u32,
    /// Frame data, padded with zeroes when constructed from shorter input.
    pub data: [u8; 8],
    /// Number of data bytes supplied to the constructor, capped at eight.
    pub dlc: u8,
}

impl Frame {
    /// Builds a frame. Short data is padded, long data is truncated, and DLC is capped at eight.
    pub fn new(pgn: u32, source: u8, destination: u8, data: impl AsRef<[u8]>) -> Self {
        let bytes = data.as_ref();
        let dlc = bytes.len().min(8);
        let mut frame_data = [0; 8];
        frame_data[..dlc].copy_from_slice(&bytes[..dlc]);
        let priority = 7u32;
        let normalized_pgn = pgn & 0x3_FFFF;
        let pf = (normalized_pgn >> 8) & 0xFF;
        let ps = if pf < 240 {
            destination as u32
        } else {
            normalized_pgn & 0xFF
        };
        let id = (priority << 26) | (((normalized_pgn >> 16) & 0x3) << 24) | (pf << 16) | (ps << 8);
        Self {
            id: id | source as u32,
            data: frame_data,
            dlc: dlc as u8,
        }
    }

    /// Sets the frame priority, returning an error when it exceeds seven.
    pub fn with_priority(mut self, priority: u8) -> Result<Self, FrameError> {
        if priority > 7 {
            return Err(FrameError::InvalidPriority(priority));
        }
        self.id = (self.id & !(0x7 << 26)) | ((priority as u32) << 26);
        Ok(self)
    }

    /// Builds a frame directly from an identifier, data bytes, and DLC.
    pub const fn from_parts(id: u32, data: [u8; 8], dlc: u8) -> Self {
        Self { id, data, dlc }
    }

    /// Returns this frame's Parameter Group Number.
    pub const fn pgn(&self) -> u32 {
        let pf = (self.id >> 16) & 0xFF;
        let ps = (self.id >> 8) & 0xFF;
        let page = (self.id >> 24) & 0x3;
        let base = (page << 16) | (pf << 8);
        if pf < 240 {
            base
        } else {
            base | ps
        }
    }

    /// Returns the source address encoded in the identifier.
    pub const fn source(&self) -> u8 {
        self.id as u8
    }

    /// Returns the destination address, or the global address for PDU2 frames.
    pub const fn destination(&self) -> u8 {
        let pf = (self.id >> 16) & 0xFF;
        if pf < 240 {
            ((self.id >> 8) & 0xFF) as u8
        } else {
            GLOBAL_ADDRESS
        }
    }

    /// Returns the three-bit priority encoded in the identifier.
    pub const fn priority(&self) -> u8 {
        ((self.id >> 26) & 0x7) as u8
    }
}

/// A 64-bit J1939-81 NAME value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Name {
    /// Packed NAME bits as defined by J1939-81.
    pub name: u64,
}

impl Name {
    /// Wraps a packed NAME value.
    pub const fn new(name: u64) -> Self {
        Self { name }
    }
    /// Returns the 21-bit identity number.
    pub const fn identity_number(self) -> u32 {
        (self.name & 0x1F_FFFF) as u32
    }
    /// Returns the 11-bit manufacturer code.
    pub const fn manufacturer_code(self) -> u16 {
        ((self.name >> 21) & 0x7FF) as u16
    }
    /// Returns the three-bit ECU instance.
    pub const fn ecu_instance(self) -> u8 {
        ((self.name >> 32) & 0x7) as u8
    }
    /// Returns the five-bit function instance.
    pub const fn function_instance(self) -> u8 {
        ((self.name >> 35) & 0x1F) as u8
    }
    /// Returns the eight-bit function code.
    pub const fn function(self) -> u8 {
        ((self.name >> 40) & 0xFF) as u8
    }
    /// Returns the reserved NAME bit.
    pub const fn reserved(self) -> bool {
        ((self.name >> 48) & 1) != 0
    }
    /// Returns the seven-bit vehicle-system code.
    pub const fn vehicle_system(self) -> u8 {
        ((self.name >> 49) & 0x7F) as u8
    }
    /// Returns the four-bit vehicle-system instance.
    pub const fn vehicle_system_instance(self) -> u8 {
        ((self.name >> 56) & 0xF) as u8
    }
    /// Returns the three-bit industry group.
    pub const fn industry_group(self) -> u8 {
        ((self.name >> 60) & 0x7) as u8
    }
    /// Returns whether the NAME supports arbitrary addresses.
    pub const fn arbitrary_address_capable(self) -> bool {
        ((self.name >> 63) & 1) != 0
    }
}

/// Caller-supplied monotonic time in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickMillis(
    /// Milliseconds elapsed from the caller's chosen time origin.
    pub u64,
);

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame(id={:#010x}, pgn={:#07x}, source={}, destination={}, dlc={})",
            self.id,
            self.pgn(),
            self.source(),
            self.destination(),
            self.dlc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_round_trips_pdu1_pgn_and_destination() {
        let frame = Frame::new(PGN_TP_CM, 33, GLOBAL_ADDRESS, [0; 8]);
        assert_eq!(frame.pgn(), PGN_TP_CM);
        assert_eq!(frame.source(), 33);
        assert_eq!(frame.destination(), GLOBAL_ADDRESS);
        assert_eq!(frame.priority(), 7);
    }

    #[test]
    fn pdu2_pgn_keeps_group_extension() {
        let frame = Frame::new(0x00_F004, 7, GLOBAL_ADDRESS, [0; 8]);
        assert_eq!(frame.pgn(), 0x00_F004);
    }

    #[test]
    fn frame_default_priority_is_seven() {
        assert_eq!(Frame::new(0xEF00, 1, GLOBAL_ADDRESS, [0; 8]).priority(), 7);
    }

    #[test]
    fn frame_custom_priority_is_preserved() {
        let frame = Frame::new(0xEF00, 1, GLOBAL_ADDRESS, [0; 8])
            .with_priority(3)
            .unwrap();
        assert_eq!(frame.priority(), 3);
    }

    #[test]
    fn frame_rejects_priority_outside_three_bits() {
        assert_eq!(
            Frame::new(0xEF00, 1, GLOBAL_ADDRESS, [0; 8]).with_priority(8),
            Err(FrameError::InvalidPriority(8))
        );
    }

    #[test]
    fn frame_truncates_data_over_eight_bytes() {
        let frame = Frame::new(0xEF00, 1, GLOBAL_ADDRESS, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(frame.dlc, 8);
        assert_eq!(frame.data, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn frame_pads_data_under_eight_bytes() {
        let frame = Frame::new(0xEF00, 1, GLOBAL_ADDRESS, [1, 2, 3]);
        assert_eq!(frame.dlc, 3);
        assert_eq!(&frame.data[3..], &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn pgn_zero_roundtrips() {
        assert_eq!(Frame::new(0, 1, 0, [0; 8]).pgn(), 0);
    }

    #[test]
    fn maximum_eighteen_bit_pgn_roundtrips() {
        assert_eq!(Frame::new(0x03_FFFF, 1, 0, [0; 8]).pgn(), 0x03_FFFF);
    }

    #[test]
    fn extended_data_page_bit_roundtrips() {
        assert_eq!(Frame::new(0x02_0000, 1, 0, [0; 8]).pgn(), 0x02_0000);
    }

    #[test]
    fn pdu2_destination_is_broadcast_even_when_argument_differs() {
        assert_eq!(
            Frame::new(0x00_FEF1, 1, 0, [0; 8]).destination(),
            GLOBAL_ADDRESS
        );
    }

    #[test]
    fn pdu1_preserves_broadcast_null_and_specific_destinations() {
        for destination in [GLOBAL_ADDRESS, 0xFE, 0x42] {
            assert_eq!(
                Frame::new(PGN_TP_CM, 1, destination, [0; 8]).destination(),
                destination
            );
        }
    }
}
