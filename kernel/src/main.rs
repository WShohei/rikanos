#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::console::Console;
use kernel::graphics::{FrameBufferConfig, Graphics, PixelColor};
use kernel::pci::{self, scan_all_bus};
use kernel::{print, println};

// set the memory allocator
use linked_list_allocator::LockedHeap;
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
// end of setting the memory allocator

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn kernel_main(c: &FrameBufferConfig) -> () {
    Graphics::initialize(*c);
    let g = Graphics::instance();
    Console::initialize(*g, PixelColor::new(255, 255, 255), PixelColor::new(0, 0, 0));

    // Clear the screen
    g.clear(&PixelColor::new(0, 0, 0));

    scan_all_bus();
    let num_devices = unsafe { pci::NUM_DEVICES };
    for i in 0..num_devices {
        let device = if let Some(device) = pci::get_device(i) {
            device
        } else {
            break;
        };
        let vendor_id = pci::read_vendor_id(device.bus, device.device, device.function);
        let class_code = pci::read_class_code(device.bus, device.device, device.function);
        println!(
            "{}.{}.{}: vend {:04x}, class {:08x}, header {:02x}",
            device.bus, device.device, device.function, vendor_id, class_code, device.header_type
        );
    }

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}
