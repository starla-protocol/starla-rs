#[derive(Clone, Copy, Debug)]
pub enum ProtocolError {
    NotFound,
    InvalidState,
    CapabilityDenied,
    IdempotencyConflict,
}
