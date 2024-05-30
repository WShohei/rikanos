pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Code {
    Success,
    Full,
    Empty,
    NoEnoughMemory,
    IndexOutOfRange,
    HostControllerNotHalted,
    InvalidSlotID,
    PortNotConnected,
    InvalidEndpointNumber,
    TransferRingNotSet,
    AlreadyAllocated,
    NotImplemented,
    InvalidDescriptor,
    BufferTooSmall,
    UnknownDevice,
    NoCorrespondingSetupStage,
    TransferFailed,
    InvalidPhase,
    UnknownXHCISpeedID,
    NoWaiter,
    NoPCIMSI,
    UnknownPixelFormat,
    NoSuchTask,
    InvalidFormat,
    FrameTooSmall,
    InvalidFile,
    IsDirectory,
    NoSuchEntry,
    FreeTypeError,
    EndpointNotInCharge,
    LastOfCode, // この列挙子は常に最後に配置する
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Error {
    code: Code,
}

impl Error {
    const fn new(code: Code) -> Self {
        Error { code }
    }
}

impl Error {
    pub const SUCCESS: Error = Error::new(Code::Success);
    pub const FULL: Error = Error::new(Code::Full);
    pub const EMPTY: Error = Error::new(Code::Empty);
    pub const NO_ENOUGH_MEMORY: Error = Error::new(Code::NoEnoughMemory);
    pub const INDEX_OUT_OF_RANGE: Error = Error::new(Code::IndexOutOfRange);
    pub const HOST_CONTROLLER_NOT_HALTED: Error = Error::new(Code::HostControllerNotHalted);
    pub const INVALID_SLOT_ID: Error = Error::new(Code::InvalidSlotID);
    pub const PORT_NOT_CONNECTED: Error = Error::new(Code::PortNotConnected);
    pub const INVALID_ENDPOINT_NUMBER: Error = Error::new(Code::InvalidEndpointNumber);
    pub const TRANSFER_RING_NOT_SET: Error = Error::new(Code::TransferRingNotSet);
    pub const ALREADY_ALLOCATED: Error = Error::new(Code::AlreadyAllocated);
    pub const NOT_IMPLEMENTED: Error = Error::new(Code::NotImplemented);
    pub const INVALID_DESCRIPTOR: Error = Error::new(Code::InvalidDescriptor);
    pub const BUFFER_TOO_SMALL: Error = Error::new(Code::BufferTooSmall);
    pub const UNKNOWN_DEVICE: Error = Error::new(Code::UnknownDevice);
    pub const NO_CORRESPONDING_SETUP_STAGE: Error = Error::new(Code::NoCorrespondingSetupStage);
    pub const TRANSFER_FAILED: Error = Error::new(Code::TransferFailed);
    pub const INVALID_PHASE: Error = Error::new(Code::InvalidPhase);
    pub const UNKNOWN_XHCI_SPEED_ID: Error = Error::new(Code::UnknownXHCISpeedID);
    pub const NO_WAITER: Error = Error::new(Code::NoWaiter);
    pub const NO_PCI_MSI: Error = Error::new(Code::NoPCIMSI);
    pub const UNKNOWN_PIXEL_FORMAT: Error = Error::new(Code::UnknownPixelFormat);
    pub const NO_SUCH_TASK: Error = Error::new(Code::NoSuchTask);
    pub const INVALID_FORMAT: Error = Error::new(Code::InvalidFormat);
    pub const FRAME_TOO_SMALL: Error = Error::new(Code::FrameTooSmall);
    pub const INVALID_FILE: Error = Error::new(Code::InvalidFile);
    pub const IS_DIRECTORY: Error = Error::new(Code::IsDirectory);
    pub const NO_SUCH_ENTRY: Error = Error::new(Code::NoSuchEntry);
    pub const FREE_TYPE_ERROR: Error = Error::new(Code::FreeTypeError);
    pub const ENDPOINT_NOT_IN_CHARGE: Error = Error::new(Code::EndpointNotInCharge);
    pub const LAST_OF_CODE: Error = Error::new(Code::LastOfCode);
}
