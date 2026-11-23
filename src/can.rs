use crate::error::AutomotiveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanId {
    Standard(u16),
    Extended(u32),
}

impl CanId {
    pub const fn standard(id: u16) -> Result<Self, AutomotiveError> {
        if id <= 0x7FF {
            Ok(Self::Standard(id))
        } else {
            Err(AutomotiveError::InvalidStandardId)
        }
    }

    pub const fn extended(id: u32) -> Result<Self, AutomotiveError> {
        if id <= 0x1FFF_FFFF {
            Ok(Self::Extended(id))
        } else {
            Err(AutomotiveError::InvalidExtendedId)
        }
    }

    pub const fn raw(self) -> u32 {
        match self {
            Self::Standard(id) => id as u32,
            Self::Extended(id) => id,
        }
    }

    pub const fn is_extended(self) -> bool {
        matches!(self, Self::Extended(_))
    }

    /// Lower numeric value represents higher arbitration priority.
    pub const fn priority_level(self) -> u8 {
        match self {
            Self::Standard(id) => (id >> 8) as u8,
            Self::Extended(id) => ((id >> 26) & 0x07) as u8,
        }
    }

    pub const fn matches_mask(self, filter: u32, mask: u32) -> bool {
        (self.raw() & mask) == (filter & mask)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameFlags {
    pub extended_dlc: bool,
    pub bit_rate_switch: bool,
    pub error_state_indicator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    pub id: CanId,
    pub dlc: u8,
    pub flags: FrameFlags,
    pub data: [u8; 8],
}

impl CanFrame {
    pub const fn new(id: CanId, payload: &[u8]) -> Result<Self, AutomotiveError> {
        if payload.len() > 8 {
            return Err(AutomotiveError::PayloadTooLarge);
        }
        let mut data = [0u8; 8];
        let mut i = 0;
        while i < payload.len() {
            data[i] = payload[i];
            i += 1;
        }
        Ok(Self {
            id,
            dlc: payload.len() as u8,
            flags: FrameFlags {
                extended_dlc: false,
                bit_rate_switch: false,
                error_state_indicator: false,
            },
            data,
        })
    }

    pub const fn empty(id: CanId) -> Self {
        Self {
            id,
            dlc: 0,
            flags: FrameFlags {
                extended_dlc: false,
                bit_rate_switch: false,
                error_state_indicator: false,
            },
            data: [0u8; 8],
        }
    }

    pub const fn with_flags(mut self, flags: FrameFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn payload(&self) -> &[u8] {
        let len = (self.dlc as usize).min(8);
        &self.data[..len]
    }
}