use crate::can::CanFrame;
use crate::error::AutomotiveError;
use crate::quad::{get_quad, set_quad, Quad, QuadArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadSignal {
    pub start_bit: u8,
    pub quad_count: u8,
}

impl QuadSignal {
    pub const fn new(start_bit: u8, quad_count: u8) -> Result<Self, AutomotiveError> {
        if quad_count == 0 {
            return Err(AutomotiveError::InvalidBitWidth);
        }
        let end_bit = (start_bit as u16) + (quad_count as u16) * 2;
        if end_bit > 64 {
            return Err(AutomotiveError::BitRangeOutOfBounds);
        }
        Ok(Self {
            start_bit,
            quad_count,
        })
    }

    pub fn decode_into_slice(
        &self,
        frame: &CanFrame,
        out: &mut [Quad],
    ) -> Result<(), AutomotiveError> {
        let count = self.quad_count as usize;
        if out.len() < count {
            return Err(AutomotiveError::IndexOutOfBounds);
        }
        let payload = frame.payload();
        let required_bytes = (((self.start_bit as usize) + count * 2) + 7) / 8;
        if payload.len() < required_bytes {
            return Err(AutomotiveError::PayloadTooLarge);
        }
        let start_quad_idx = (self.start_bit as usize) / 2;
        for (i, slot) in out.iter_mut().take(count).enumerate() {
            *slot = get_quad(payload, start_quad_idx + i)?;
        }
        Ok(())
    }

    pub fn encode_from_slice(
        &self,
        frame: &mut CanFrame,
        values: &[Quad],
    ) -> Result<(), AutomotiveError> {
        let count = self.quad_count as usize;
        if values.len() < count {
            return Err(AutomotiveError::IndexOutOfBounds);
        }
        let required_bytes = (((self.start_bit as usize) + count * 2) + 7) / 8;
        if required_bytes > 8 {
            return Err(AutomotiveError::PayloadTooLarge);
        }
        if (frame.dlc as usize) < required_bytes {
            frame.dlc = required_bytes as u8;
        }
        let start_quad_idx = (self.start_bit as usize) / 2;
        for (i, &val) in values.iter().take(count).enumerate() {
            set_quad(&mut frame.data[..required_bytes], start_quad_idx + i, val)?;
        }
        Ok(())
    }

    pub fn decode_array<const PACKED_BYTES: usize>(
        &self,
        frame: &CanFrame,
    ) -> Result<QuadArray<PACKED_BYTES>, AutomotiveError> {
        let count = self.quad_count as usize;
        if QuadArray::<PACKED_BYTES>::CAPACITY < count {
            return Err(AutomotiveError::BufferTooSmall);
        }
        let mut arr = QuadArray::<PACKED_BYTES>::new(Quad::Q00);
        let payload = frame.payload();
        let required_bytes = (((self.start_bit as usize) + count * 2) + 7) / 8;
        if payload.len() < required_bytes {
            return Err(AutomotiveError::PayloadTooLarge);
        }
        let start_quad_idx = (self.start_bit as usize) / 2;
        for i in 0..count {
            let q = get_quad(payload, start_quad_idx + i)?;
            arr.set(i, q)?;
        }
        Ok(arr)
    }
    
}

/// Quantizes a `u8` value (`0..=255`) into a 4-level quaternary symbol (`00`, `01`, `10`, `11`).
pub const fn quantize_u8_to_quad(value: u8) -> Quad {
    match value >> 6 {
        0 => Quad::Q00,
        1 => Quad::Q01,
        2 => Quad::Q10,
        _ => Quad::Q11,
    }
}

/// Expands a quaternary symbol back to a representative `u8` midpoint (`0..=255`).
pub const fn dequantize_quad_to_u8(q: Quad) -> u8 {
    match q {
        Quad::Q00 => 0,
        Quad::Q01 => 85,
        Quad::Q10 => 170,
        Quad::Q11 => 255,
    }
}