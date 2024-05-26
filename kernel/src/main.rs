#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer_base: usize, frame_buffer_size: usize) -> () {
    unsafe {
        let frame_buffer_base = frame_buffer_base as *mut u8;
        
        for i in 0..frame_buffer_size {
            frame_buffer_base.add(i).write_volatile(255);
        }

        loop {
            asm!("hlt");
        }
    }
}
