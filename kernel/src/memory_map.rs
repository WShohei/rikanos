pub use uefi::table::boot::{MemoryMap, MemoryType};

pub struct MemoryMapDescriptor {
    memory_type: MemoryType,
    physical_start: u64,
    virtual_start: u64,
    number_of_pages: u64,
    attribute: u64,
}
