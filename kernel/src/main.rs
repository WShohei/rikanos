#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::console::Console;
use kernel::graphics::{FrameBufferConfig, Graphics, PixelColor, Vector2D};
use kernel::mouse::{self, MouseCursor};
use kernel::pci::Device;
use kernel::usb::{self, XhciController};
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
    let graphics = Graphics::instance();
    Console::initialize(
        *graphics,
        PixelColor::new(255, 255, 255),
        PixelColor::new(237, 18, 45),
    );
    Device::initialize();
    MouseCursor::initialize(*graphics, Vector2D::<usize>::new(200, 300));
    usb::register_mouse_observer(mouse_observer);

    for i in 0..Device::num_devices() {
        let device = Device::get_device(i).unwrap();
        println!(
            "{}.{}.{}: vendor_id = {:#x}, device_id = {:#x}, class_code = {:#x}",
            device.bus,
            device.device,
            device.function,
            device.read_vendor_id(),
            device.read_device_id(),
            device.read_class_code()
        )
    }

    let xhc_dev = if let Some(xhc_dev) = Device::find_xhc_device() {
        xhc_dev
    } else {
        println!("xHC not found");
        loop {}
    };
    let xhc_bar = xhc_dev.read_bar(0).unwrap();
    println!("xHC BAR0 = {:#x}", xhc_bar);
    let xhc_mmio_base = xhc_bar & 0xffff_ffff_ffff_fff0;
    println!("xHC MMIO Base = {:#x}", xhc_mmio_base);

    let xhc = XhciController::new(xhc_mmio_base);
    let xhc_dev_vendor_id = xhc_dev.read_vendor_id();
    println!("xHC Vendor ID = {:#x}", xhc_dev_vendor_id);
    if xhc_dev_vendor_id == 0x8086 {
        xhc_dev.switch_ehci2xhci().unwrap();
        println!("Intel xHC detected");
    }
    let e = xhc.initialize();
    println!("XhciController.initialize finished. code = {:?}", e);

    println!("Start running xHC");
    xhc.run().unwrap();

    xhc.configure_port();
    loop {
        xhc.process_event().unwrap();
    }
}

extern "C" fn mouse_observer(displacement_x: i8, displacement_y: i8) {
    let mouse_cursor = MouseCursor::instance();
    mouse_cursor.move_relative(displacement_x, displacement_y);
}
