#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::vec;
use core::result::Result;
use log::info;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileMode, FileType::Regular, RegularFile,
};
use uefi::{prelude::*, Error};

use linked_list_allocator::LockedHeap;
const HEAP_SIZE: usize = 64 * 1024; // 64 KiB
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn open_root_dir(bs: &BootServices, handle: Handle) -> Result<Directory, Error> {
    let mut sfs = bs.get_image_file_system(handle)?;
    let root = sfs.open_volume()?;
    Ok(root)
}

fn save_memory_map(bs: &BootServices, mut file: RegularFile) -> Result<(), Error> {
    let mmap_buf = &mut vec![0; 4096 * 4];
    let mmap_buf: &mut [u8] = &mut mmap_buf.as_mut_slice();
    let mut mmap = bs.memory_map(mmap_buf)?;
    mmap.sort();
    let mmap_iter = mmap.entries();

    file.write("Index, Type, PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap();

    for (i, m) in mmap_iter.enumerate() {
        file.write(
            format!(
                "{}, {:?}, {}, {}, {:?}\n",
                i, m.ty, m.phys_start, m.page_count, m.att
            )
            .as_bytes(),
        )
        .unwrap();
    }
    file.close();
    Ok(())
}

#[entry]
#[allow(dead_code)]
#[allow(unreachable_code)]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    info!("Hello, UEFI!");

    // Initialize the heap
    info!("Initializing heap...");
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_ptr() as *mut u8, HEAP_SIZE);
    }
    info!("Heap initialized");

    let bs: &BootServices = system_table.boot_services();

    let mut root = if let Ok(root) = open_root_dir(bs, handle) {
        root
    } else {
        return Status::ABORTED;
    };

    let mmap_file_handle = root
        .open(
            cstr16!("mmap"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap();
    let mmap_file_handle = mmap_file_handle.into_type().unwrap();

    if let Regular(mmap_file_handle) = mmap_file_handle {
        save_memory_map(bs, mmap_file_handle).unwrap();
    }

    info!("Wrote memory map to mmap file");

    loop {}

    Status::SUCCESS
}
