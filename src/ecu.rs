use crate::can::{CanFrame, CanId};
use crate::error::AutomotiveError;
use crate::quad::{Quad, QuadArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EcuState {
    Off = 0b00,
    Standby = 0b01,
    Active = 0b10,
    Fault = 0b11,
}

impl From<Quad> for EcuState {
    fn from(q: Quad) -> Self {
        match q {
            Quad::Q00 => Self::Off,
            Quad::Q01 => Self::Standby,
            Quad::Q10 => Self::Active,
            Quad::Q11 => Self::Fault,
        }
    }
}

impl From<EcuState> for Quad {
    fn from(s: EcuState) -> Self {
        match s {
            EcuState::Off => Quad::Q00,
            EcuState::Standby => Quad::Q01,
            EcuState::Active => Quad::Q10,
            EcuState::Fault => Quad::Q11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcuCommand {
    Sleep = 0b00,
    Wake = 0b01,
    Run = 0b10,
    ResetFaults = 0b11,
}

impl EcuCommand {
    pub const fn from_quad(q: Quad) -> Self {
        match q {
            Quad::Q00 => Self::Sleep,
            Quad::Q01 => Self::Wake,
            Quad::Q10 => Self::Run,
            Quad::Q11 => Self::ResetFaults,
        }
    }

    pub const fn to_quad(self) -> Quad {
        match self {
            Self::Sleep => Quad::Q00,
            Self::Wake => Quad::Q01,
            Self::Run => Quad::Q10,
            Self::ResetFaults => Quad::Q11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EcuNode {
    pub id: CanId,
    pub states: QuadArray<4>,
    pub faults: QuadArray<4>,
}

impl EcuNode {
    pub const fn new(id: CanId) -> Self {
        Self {
            id,
            states: QuadArray::new(Quad::Q00),
            faults: QuadArray::new(Quad::Q00),
        }
    }

    pub fn get_slot(&self, slot: usize) -> Result<EcuState, AutomotiveError> {
        if slot >= 4 {
            return Err(AutomotiveError::SlotOutOfBounds);
        }
        Ok(EcuState::from(self.states.get(slot)?))
    }

    pub fn set_slot(&mut self, slot: usize, state: EcuState) -> Result<(), AutomotiveError> {
        if slot >= 4 {
            return Err(AutomotiveError::SlotOutOfBounds);
        }
        self.states.set(slot, Quad::from(state))
    }

    pub fn get_fault_level(&self, slot: usize) -> Result<Quad, AutomotiveError> {
        if slot >= 4 {
            return Err(AutomotiveError::SlotOutOfBounds);
        }
        self.faults.get(slot)
    }

    pub fn set_fault_level(&mut self, slot: usize, level: Quad) -> Result<(), AutomotiveError> {
        if slot >= 4 {
            return Err(AutomotiveError::SlotOutOfBounds);
        }
        self.faults.set(slot, level)?;
        if level == Quad::Q11 {
            self.set_slot(slot, EcuState::Fault)?;
        }
        Ok(())
    }

    pub fn apply_command(&mut self, slot: usize, cmd: EcuCommand) -> Result<(), AutomotiveError> {
        if slot >= 4 {
            return Err(AutomotiveError::SlotOutOfBounds);
        }
        match cmd {
            EcuCommand::Sleep => self.set_slot(slot, EcuState::Off),
            EcuCommand::Wake => self.set_slot(slot, EcuState::Standby),
            EcuCommand::Run => self.set_slot(slot, EcuState::Active),
            EcuCommand::ResetFaults => {
                self.faults.set(slot, Quad::Q00)?;
                if self.get_slot(slot)? == EcuState::Fault {
                    self.set_slot(slot, EcuState::Standby)?;
                }
                Ok(())
            }
        }
    }

    pub fn has_any_fault(&self) -> bool {
        self.faults.as_bytes()[0] != 0
    }

    pub fn clear_all_faults(&mut self) {
        self.faults = QuadArray::new(Quad::Q00);
    }

    pub fn to_status_frame(&self) -> Result<CanFrame, AutomotiveError> {
        CanFrame::new(
            self.id,
            &[self.states.as_bytes()[0], self.faults.as_bytes()[0]],
        )
    }

    pub fn from_status_frame(frame: &CanFrame) -> Result<Self, AutomotiveError> {
        let payload = frame.payload();
        if payload.is_empty() {
            return Err(AutomotiveError::EmptyPayload);
        }
        let states = QuadArray::<4>::from_bytes(&[payload[0]])?;
        let faults = if payload.len() >= 2 {
            QuadArray::<4>::from_bytes(&[payload[1]])?
        } else {
            QuadArray::new(Quad::Q00)
        };
        Ok(Self {
            id: frame.id,
            states,
            faults,
        })
    }
}