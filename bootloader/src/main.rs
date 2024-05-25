#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::{slice, vec::Vec};
use core::result::Result;
use log::info;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileType::Regular, RegularFile,
};
use uefi::table::boot::{AllocateType, MemoryMap, MemoryType};
use uefi::{prelude::*, Error};

// set the memory allocator
use linked_list_allocator::LockedHeap;
const HEAP_SIZE: usize = 64 * 1024; // 64 KiB
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
// end of setting the memory allocator

const BASE_ADDR: u64 = 0x100000;

fn open_root_dir(bs: &BootServices, handle: Handle) -> Result<Directory, Error> {
    let mut sfs = bs.get_image_file_system(handle)?;
    let root = sfs.open_volume()?;
    Ok(root)
}

fn get_memory_map<'a>(
    bs: &'a BootServices,
    mmap_buf: &'a mut [u8],
) -> Result<MemoryMap<'a>, Error> {
    let mut mmap = bs.memory_map(mmap_buf)?;
    mmap.sort();

    Ok(mmap)
}

fn save_memory_map<'a>(
    bs: &'a BootServices,
    file: &mut RegularFile,
    mmap_buf: &'a mut [u8],
) -> Result<MemoryMap<'a>, Error> {
    let mmap = get_memory_map(bs, mmap_buf).unwrap();
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
    Ok(mmap)
}

fn load_kernel_file(bs: &BootServices, kernel_file_handle: &mut RegularFile) -> Result<(), Error> {
    let mut file_info_buf: Vec<u8> = Vec::new();
    let info_size = kernel_file_handle
        .get_info::<FileInfo>(&mut file_info_buf)
        .unwrap_err() // This should fail because the buffer is too small
        .data()
        .unwrap();
    file_info_buf.resize(info_size, 0);
    let info = kernel_file_handle
        .get_info::<FileInfo>(&mut file_info_buf)
        .unwrap();
    let kernel_file_size = info.file_size();

    bs.allocate_pages(
        AllocateType::Address(BASE_ADDR),
        MemoryType::LOADER_DATA,
        (kernel_file_size as usize + 0xfff) / 0x1000,
    )?;
    unsafe {
        let addr = BASE_ADDR as *mut u8;
        let buffer = slice::from_raw_parts_mut(addr, kernel_file_size as usize);
        kernel_file_handle.read(buffer)?;
    }

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
    // End of heap initialization

    // Open the root directory
    let bs: &BootServices = system_table.boot_services();

    let mut root = if let Ok(root) = open_root_dir(bs, handle) {
        root
    } else {
        return Status::ABORTED;
    };
    // End of opening the root directory

    // Save the memory map to a file
    let mmap_file_handle = root
        .open(
            cstr16!("mmap"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap();
    let mmap_file_handle = mmap_file_handle.into_type().unwrap();

    let mmap_buf = &mut vec![0; 4096 * 4];
    let mmap_buf: &mut [u8] = &mut mmap_buf.as_mut_slice();
    if let Regular(mut mmap_file_handle) = mmap_file_handle {
        let _ = save_memory_map(bs, &mut mmap_file_handle, mmap_buf).unwrap();
        mmap_file_handle.flush().unwrap();
        mmap_file_handle.close();
    } else {
        info!("Failed to open mmap file");
        return Status::ABORTED;
    }

    info!("Wrote memory map to mmap file");
    // End of saving the memory map to a file

    // Load the kernel file
    let kernel_file_handle = root
        .open(
            cstr16!("mikan-kernel"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .unwrap();
    let kernel_file_handle = kernel_file_handle.into_type().unwrap();

    if let Regular(mut kernel_file_handle) = kernel_file_handle {
        load_kernel_file(bs, &mut kernel_file_handle).unwrap();
    } else {
        info!("Failed to open kernel file");
        return Status::ABORTED;
    }
    // End of loading the kernel file

    // Exit boot services
    info!("Exiting boot services and jumping to the kernel...");
    let _ = system_table.exit_boot_services(MemoryType::RESERVED);
    // End of exiting boot services

    // Jump to the kernel
    let entry_point_addr = unsafe {
        let addr_ptr = (BASE_ADDR + 24) as *const u8;
        let addr_slice = core::slice::from_raw_parts(addr_ptr, 8);
        u64::from_le_bytes([
            addr_slice[0],
            addr_slice[1],
            addr_slice[2],
            addr_slice[3],
            addr_slice[4],
            addr_slice[5],
            addr_slice[6],
            addr_slice[7],
        ])
    };
    let entry_point: extern "C" fn() -> ! = unsafe { core::mem::transmute(entry_point_addr) };

    entry_point();
    // End of jumping to the kernel

    Status::SUCCESS
}
