#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::graphics::{FrameBufferConfig, Graphics, PixelColor};
use kernel::console::Console;
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
    let g = Graphics::new(*c).clone();
    Console::initialize(g, PixelColor::new(255, 255, 255), PixelColor::new(0, 0, 0));
    
    // Clear the screen
    g.clear(&PixelColor::new(0, 0, 0));

    // Write a string to the screen
    print!("Hello, world!");

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}
