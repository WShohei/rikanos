extern "C" {
    fn UsbXhciController(xhc_mmio_base: u64) -> *mut XhciControllerImpl;
    fn UsbXhciController_initialize(c_impl: *mut XhciControllerImpl) -> i32;
    fn UsbXhciController_run(c_impl: *mut XhciControllerImpl) -> i32;
    fn UsbXhciController_configurePort(c_impl: *mut XhciControllerImpl);
    fn UsbXhciController_ProcessXhcEvent(c_impl: *mut XhciControllerImpl) -> i32;
    fn UsbXhciController_PrimaryEventRing_HasFront(c_impl: *mut XhciControllerImpl) -> bool;

    /// ref: https://doc.rust-lang.org/nomicon/ffi.html#targeting-callbacks-to-rust-objects
    fn RegisterMouseObserver(
        cb: extern "C" fn(buttons: u8, displacement_x: i8, displacement_y: i8),
    );

    fn RegisterKeyboardObserver(cb: extern "C" fn(modifier: u8, keycode: u8, press: bool));
}

