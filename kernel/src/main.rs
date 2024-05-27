#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn kernel_main(frame_buffer_base: usize, frame_buffer_size: usize) -> () {
    unsafe {
        for i in 0..frame_buffer_size {
            let addr = frame_buffer_base + i;
            core::ptr::write_volatile(addr as *mut u8, (i % 256) as u8);
        }

        //loop {
        //    asm!("hlt");
        //}
    }
}
