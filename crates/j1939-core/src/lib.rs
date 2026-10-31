#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

pub const PGN_TP_DT: u32 = 0x00_EB00;
pub const PGN_TP_CM: u32 = 0x00_EC00;
pub const PGN_ADDRESS_CLAIMED: u32 = 0x00_EE00;
pub const PGN_REQUEST: u32 = 0x00_EA00;
pub const GLOBAL_ADDRESS: u8 = 0xFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame {
    pub id: u32,
    pub data: [u8; 8],
    pub dlc: u8,
}

impl Frame {
    pub const fn new(pgn: u32, source: u8, destination: u8, data: [u8; 8]) -> Self {
        let priority = 6u32;
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
            data,
            dlc: 8,
        }
    }

    pub const fn from_parts(id: u32, data: [u8; 8], dlc: u8) -> Self {
        Self { id, data, dlc }
    }

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

    pub const fn source(&self) -> u8 {
        self.id as u8
    }

    pub const fn destination(&self) -> u8 {
        let pf = (self.id >> 16) & 0xFF;
        if pf < 240 {
            ((self.id >> 8) & 0xFF) as u8
        } else {
            GLOBAL_ADDRESS
        }
    }

    pub const fn priority(&self) -> u8 {
        ((self.id >> 26) & 0x7) as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Name {
    pub name: u64,
}

impl Name {
    pub const fn new(name: u64) -> Self {
        Self { name }
    }
    pub const fn identity_number(self) -> u32 {
        (self.name & 0x1F_FFFF) as u32
    }
    pub const fn manufacturer_code(self) -> u16 {
        ((self.name >> 21) & 0x7FF) as u16
    }
    pub const fn ecu_instance(self) -> u8 {
        ((self.name >> 32) & 0x7) as u8
    }
    pub const fn function_instance(self) -> u8 {
        ((self.name >> 35) & 0x1F) as u8
    }
    pub const fn function(self) -> u8 {
        ((self.name >> 40) & 0xFF) as u8
    }
    pub const fn reserved(self) -> bool {
        ((self.name >> 48) & 1) != 0
    }
    pub const fn vehicle_system(self) -> u8 {
        ((self.name >> 49) & 0x7F) as u8
    }
    pub const fn vehicle_system_instance(self) -> u8 {
        ((self.name >> 56) & 0xF) as u8
    }
    pub const fn industry_group(self) -> u8 {
        ((self.name >> 60) & 0x7) as u8
    }
    pub const fn arbitrary_address_capable(self) -> bool {
        ((self.name >> 63) & 1) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickMillis(pub u64);

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
        assert_eq!(frame.priority(), 6);
    }

    #[test]
    fn pdu2_pgn_keeps_group_extension() {
        let frame = Frame::new(0x00_F004, 7, GLOBAL_ADDRESS, [0; 8]);
        assert_eq!(frame.pgn(), 0x00_F004);
    }
}
