use crate::can::CanFrame;
use crate::ecu::EcuNode;
use crate::error::AutomotiveError;
use crate::quad::Quad;

pub trait CanBus {
    fn transmit(&mut self, frame: &CanFrame) -> Result<(), AutomotiveError>;
    fn receive(&mut self) -> Result<CanFrame, AutomotiveError>;
    fn is_ready(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFilter {
    pub id_filter: u32,
    pub id_mask: u32,
}

impl CanFilter {
    pub const fn accept_all() -> Self {
        Self {
            id_filter: 0,
            id_mask: 0,
        }
    }

    pub const fn exact(raw_id: u32) -> Self {
        Self {
            id_filter: raw_id,
            id_mask: 0x1FFF_FFFF,
        }
    }

    pub const fn matches(&self, frame: &CanFrame) -> bool {
        frame.id.matches_mask(self.id_filter, self.id_mask)
    }
}

pub struct EcuRegistry<'a> {
    nodes: &'a mut [EcuNode],
}

impl<'a> EcuRegistry<'a> {
    pub fn new(nodes: &'a mut [EcuNode]) -> Self {
        Self { nodes }
    }

    pub fn find_mut(&mut self, raw_id: u32) -> Option<&mut EcuNode> {
        self.nodes.iter_mut().find(|node| node.id.raw() == raw_id)
    }

    pub fn ingest_frame(&mut self, frame: &CanFrame) -> Result<bool, AutomotiveError> {
        let raw_id = frame.id.raw();
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id.raw() == raw_id) {
            *node = EcuNode::from_status_frame(frame)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn broadcast_statuses<B: CanBus>(&self, bus: &mut B) -> Result<usize, AutomotiveError> {
        if !bus.is_ready() {
            return Err(AutomotiveError::BusNotReady);
        }
        let mut sent = 0;
        for node in self.nodes.iter() {
            let frame = node.to_status_frame()?;
            bus.transmit(&frame)?;
            sent += 1;
        }
        Ok(sent)
    }
}

/// Tracks 4-step sequence numbers (`00` -> `01` -> `10` -> `11`) across multi-frame transfers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadSequenceTracker {
    expected: Quad,
}

impl QuadSequenceTracker {
    pub const fn new() -> Self {
        Self {
            expected: Quad::Q00,
        }
    }

    pub const fn expected(&self) -> Quad {
        self.expected
    }

    pub fn advance(&mut self, received: Quad) -> Result<Quad, AutomotiveError> {
        if received != self.expected {
            return Err(AutomotiveError::SequenceMismatch);
        }
        let current = self.expected;
        self.expected = Quad::from_bits(current.as_bits().wrapping_add(1) & 0b11);
        Ok(current)
    }

    pub fn reset(&mut self) {
        self.expected = Quad::Q00;
    }
}