use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Cell not found: {0}")]
    CellNotFound(String),

    #[error("Cell already exists: {0}")]
    CellAlreadyExists(String),

    #[error("Cell already running: {0}")]
    CellAlreadyRunning(String),

    #[error("Cell not running: {0}")]
    CellNotRunning(String),

    #[error("Invalid state transition: from {from:?} to {to:?}")]
    InvalidTransition { from: String, to: String },

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Capability expired: {0}")]
    CapabilityExpired(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    #[error("Protocol not supported: {0}")]
    ProtocolNotSupported(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Partition not found: {0}")]
    PartitionNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task already completed: {0}")]
    TaskAlreadyCompleted(String),

    #[error("Memory region not found: {0}")]
    MemoryRegionNotFound(String),

    #[error("Memory capacity exceeded: {0}")]
    MemoryCapacityExceeded(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, Error>;
