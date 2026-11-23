#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomotiveError {
    InvalidStandardId,
    InvalidExtendedId,
    PayloadTooLarge,
    EmptyPayload,
    IndexOutOfBounds,
    SlotOutOfBounds,
    BitRangeOutOfBounds,
    InvalidBitWidth,
    BusTransmitFailed,
    BusReceiveEmpty,
    BusNotReady,
    SequenceMismatch,
    UnexpectedFrameType,
    BufferTooSmall,
}