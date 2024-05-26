#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::{slice, vec::Vec};
use core::result::Result;
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

fn open_gop<'a>(bs: &'a BootServices) -> Result<ScopedProtocol<'a, GraphicsOutput>, Error> {
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
        kernel_file_handle.set_position(0)?;
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
        info!("Kernel file loaded");
    } else {
        info!("Failed to open kernel file");
        return Status::ABORTED;
    }
    //End of loading the kernel file

    //Open the GOP
    let mut gop = if let Ok(gop) = open_gop(bs) {
        info!("GOP opened");
        gop
    } else {
        info!("Failed to open GOP");
        return Status::ABORTED;
    };
    let gop_frame_base = gop.frame_buffer().as_mut_ptr() as usize;
    let gop_frame_size = gop.frame_buffer().size() as usize;
    let gop_mode_info = gop.current_mode_info();
    let gop_width = gop_mode_info.resolution().0 as usize;
    let gop_height = gop_mode_info.resolution().1 as usize;
    let gop_pixel_format = gop_mode_info.pixel_format();
    let gop_stride = gop_mode_info.stride() as usize;
    info!("GOP resolution: {}x{}", gop_width, gop_height);
    info!("GOP pixel format: {:?}", gop_pixel_format);
    info!("GOP stride: {}", gop_stride);
    info!(
        "GOP frame buffer: 0x{:x}-0x{:x}, size: {} bytes",
        gop_frame_base,
        gop_frame_base + gop_frame_size,
        gop_frame_size
    );
    unsafe {
        for i in 0..gop_frame_size {
            let addr = gop_frame_base + i;
            core::ptr::write_volatile(addr as *mut u8, 255);
        }
    }
    // End of opening the GOP

    // Exit boot services
    //let _ = system_table.exit_boot_services(MemoryType::RESERVED);
    // End of exiting boot services

    // Jump to the kernel
    let entry_point_addr: usize = unsafe {
        let addr_ptr = (BASE_ADDR + 24) as *const usize;
        *addr_ptr
    };
    info!("Kernel entry point: 0x{:x}", entry_point_addr);
    let entry_point: extern "C" fn(usize, usize) -> ! = unsafe {
        core::mem::transmute::<usize, extern "C" fn(usize, usize) -> !>(entry_point_addr)
    };

    entry_point(gop_frame_base, gop_frame_size);
    //End of jumping to the kernel

    Status::SUCCESS
}
