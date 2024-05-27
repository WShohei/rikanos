#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use core::result::Result;
use elf_rs::*;
use log::info;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileType::Regular, RegularFile,
};
use uefi::table::boot::{
    AllocateType, MemoryMap, MemoryType, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol,
};
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

fn open_gop(bs: &BootServices) -> Result<ScopedProtocol<GraphicsOutput>, Error> {
    info!("Opening GOP...");
    let gop_handle = if let Ok(gop_handle) = bs.get_handle_for_protocol::<GraphicsOutput>() {
        gop_handle
    } else {
        panic!("Failed to locate GOP handle");
    };
    let params = OpenProtocolParams {
        handle: gop_handle,
        agent: bs.image_handle(),
        controller: Some(gop_handle),
    };
    info!("GOP handle obtained");
    //let gop = if let Ok(gop) = bs.open_protocol_exclusive::<GraphicsOutput>(gop_handle) {
    //    gop
    //} else {
    //    panic!("Failed to open GOP");
    //};
    unsafe {
        let gop = if let Ok(gop) =
            bs.open_protocol::<GraphicsOutput>(params, OpenProtocolAttributes::GetProtocol)
        {
            gop
        } else {
            panic!("Failed to open GOP");
        };
        let handle_buffer = gop_handle.as_ptr() as *mut u8;
        bs.free_pool(handle_buffer)?;
        Ok(gop)
    }
}

fn load_kernel_file(bs: &BootServices, mut kernel_file_handle: RegularFile) -> Result<(), Error> {
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

    let kernel_buffer = bs
        .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size as usize)
        .unwrap();
    let kernel_buffer =
        unsafe { core::slice::from_raw_parts_mut(kernel_buffer, kernel_file_size as usize) };

    kernel_file_handle.read(kernel_buffer).unwrap();
    kernel_file_handle.close();

    let elf = match Elf::from_bytes(kernel_buffer).unwrap() {
        Elf::Elf64(elf) => elf,
        Elf::Elf32(_) => panic!("32-bit ELF not supported"),
    };

    let mut kernel_first = u64::max_value();
    let mut kernel_last = u64::min_value();
    for ph in elf.program_header_iter() {
        if ph.ph_type() == ProgramType::LOAD {
            let start = ph.vaddr() as u64;
            let end = start + ph.memsz() as u64;
            kernel_first = core::cmp::min(kernel_first, start);
            kernel_last = core::cmp::max(kernel_last, end);
        }
    }
    let kernel_first = kernel_first / 0x1000 * 0x1000; // Round down to the nearest page
    let num_pages = (kernel_last - kernel_first + 0xfff) as usize / 0x1000;

    bs.allocate_pages(
        AllocateType::Address(kernel_first),
        MemoryType::LOADER_DATA,
        num_pages,
    )
    .unwrap();

    for ph in elf.program_header_iter() {
        if ph.ph_type() == ProgramType::LOAD {
            let start = ph.vaddr() as u64;
            let offset = ph.offset() as usize;
            let size = ph.filesz() as usize;
            let buffer = &kernel_buffer[offset..offset + size];
            unsafe {
                core::ptr::copy_nonoverlapping(buffer.as_ptr(), start as *mut u8, size);
            }
            if size < ph.memsz() as usize {
                unsafe {
                    core::ptr::write_bytes((start + size as u64) as *mut u8, 0, ph.memsz() as usize - size);
                }
            }
        }
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
    let bs = system_table.boot_services();

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
            cstr16!("kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .unwrap();
    let kernel_file_handle = kernel_file_handle.into_type().unwrap();

    if let Regular(kernel_file_handle) = kernel_file_handle {
        load_kernel_file(bs, kernel_file_handle).unwrap();
        info!("Kernel file loaded");
    } else {
        info!("Failed to open kernel file");
        return Status::ABORTED;
    }
    //End of loading the kernel file

    //Open the GOP
    match open_gop(bs) {
        Ok(mut gop) => {
            info!("GOP opened");
            let gop_frame_base = gop.frame_buffer().as_mut_ptr().clone() as usize;
            let gop_frame_size = gop.frame_buffer().size().clone() as usize;
            info!(
                "GOP frame buffer: 0x{:x}-0x{:x}, size: {} bytes",
                gop_frame_base,
                gop_frame_base + gop_frame_size,
                gop_frame_size
            );
            //unsafe {
            //    for i in 0..gop_frame_size {
            //        let addr = gop_frame_base + i;
            //        core::ptr::write_volatile(addr as *mut u8, 255);
            //    }
            //}

            // Jump to the kernel
            info!("Jumping to the kernel...");
            let entry_point_addr: usize = unsafe {
                let addr_ptr = (BASE_ADDR + 24) as *const usize;
                *addr_ptr
            };
            info!("Kernel entry point: 0x{:x}", entry_point_addr);
            let entry_point_addr = entry_point_addr as *const ();
            let entry_point: extern "efiapi" fn(usize, usize) -> () = unsafe {
                core::mem::transmute::<*const (), extern "efiapi" fn(usize, usize) -> ()>(
                    entry_point_addr,
                )
            };

            //let _ = system_table.exit_boot_services(MemoryType::RESERVED); // Exit boot services

            entry_point(gop_frame_base, gop_frame_size);
        }
        Err(_) => {
            info!("Failed to open GOP");
            return Status::ABORTED;
        }
    }
    // End of opening the GOP

    Status::SUCCESS
}
