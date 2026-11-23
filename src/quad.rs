use crate::error::AutomotiveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Quad {
    Q00 = 0b00,
    Q01 = 0b01,
    Q10 = 0b10,
    Q11 = 0b11,
}

impl Quad {
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => Self::Q00,
            0b01 => Self::Q01,
            0b10 => Self::Q10,
            _ => Self::Q11,
        }
    }

    #[inline]
    pub const fn as_bits(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_bool_pair(high: bool, low: bool) -> Self {
        Self::from_bits(((high as u8) << 1) | (low as u8))
    }

    #[inline]
    pub const fn to_bool_pair(self) -> (bool, bool) {
        let b = self.as_bits();
        ((b & 0b10) != 0, (b & 0b01) != 0)
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        matches!(self, Self::Q00)
    }

    #[inline]
    pub const fn is_max(self) -> bool {
        matches!(self, Self::Q11)
    }
}

#[inline]
pub fn get_quad(buf: &[u8], index: usize) -> Result<Quad, AutomotiveError> {
    let byte_idx = index / 4;
    let shift = (index % 4) * 2;
    let byte = buf.get(byte_idx).ok_or(AutomotiveError::IndexOutOfBounds)?;
    Ok(Quad::from_bits(byte >> shift))
}

#[inline]
pub fn set_quad(buf: &mut [u8], index: usize, value: Quad) -> Result<(), AutomotiveError> {
    let byte_idx = index / 4;
    let shift = (index % 4) * 2;
    let byte = buf
        .get_mut(byte_idx)
        .ok_or(AutomotiveError::IndexOutOfBounds)?;
    let mask = !(0b11u8 << shift);
    *byte = (*byte & mask) | (value.as_bits() << shift);
    Ok(())
}

pub fn read_quads(
    buf: &[u8],
    start_index: usize,
    out: &mut [Quad],
) -> Result<(), AutomotiveError> {
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = get_quad(buf, start_index + i)?;
    }
    Ok(())
}

pub fn write_quads(
    buf: &mut [u8],
    start_index: usize,
    values: &[Quad],
) -> Result<(), AutomotiveError> {
    for (i, &val) in values.iter().enumerate() {
        set_quad(buf, start_index + i, val)?;
    }
    Ok(())
}

pub fn max_quad(buf: &[u8], start_index: usize, count: usize) -> Result<Quad, AutomotiveError> {
    let mut best = Quad::Q00;
    for i in 0..count {
        let q = get_quad(buf, start_index + i)?;
        if q.as_bits() > best.as_bits() {
            best = q;
        }
    }
    Ok(best)
}

/// Stores up to `PACKED_BYTES * 4` quaternary values in `PACKED_BYTES` bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadArray<const PACKED_BYTES: usize> {
    bytes: [u8; PACKED_BYTES],
}

impl<const PACKED_BYTES: usize> QuadArray<PACKED_BYTES> {
    pub const BYTES: usize = PACKED_BYTES;
    pub const CAPACITY: usize = PACKED_BYTES * 4;

    pub const fn new(fill: Quad) -> Self {
        let mut b = 0u8;
        let mut slot = 0;
        while slot < 4 {
            b |= fill.as_bits() << (slot * 2);
            slot += 1;
        }
        Self {
            bytes: [b; PACKED_BYTES],
        }
    }

    pub fn get(&self, index: usize) -> Result<Quad, AutomotiveError> {
        if index >= Self::CAPACITY {
            return Err(AutomotiveError::IndexOutOfBounds);
        }
        get_quad(&self.bytes, index)
    }

    pub fn set(&mut self, index: usize, value: Quad) -> Result<(), AutomotiveError> {
        if index >= Self::CAPACITY {
            return Err(AutomotiveError::IndexOutOfBounds);
        }
        set_quad(&mut self.bytes, index, value)
    }

    pub const fn as_bytes(&self) -> &[u8; PACKED_BYTES] {
        &self.bytes
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, AutomotiveError> {
        if slice.len() < PACKED_BYTES {
            return Err(AutomotiveError::BufferTooSmall);
        }
        let mut bytes = [0u8; PACKED_BYTES];
        bytes.copy_from_slice(&slice[..PACKED_BYTES]);
        Ok(Self { bytes })
    }
}