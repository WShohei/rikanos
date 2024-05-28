#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::graphics::{FrameBufferConfig, Graphics, PixelColor};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn kernel_main(c: &FrameBufferConfig) -> () {
    let g = Graphics::new(*c);
    unsafe {
        for y in 0..c.mode_info.resolution().1 {
            for x in 0..c.mode_info.resolution().0 {
                (g.pixel_writer)(c, x, y, PixelColor::new(0, 255, 0));
            }
        }
        loop {
            asm!("hlt");
        }
    }
}
