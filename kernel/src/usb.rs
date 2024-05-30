use crate::error::Error;
use crate::{print, println};

extern "C" {
    fn UsbXhciController(xhc_mmio_base: u64) -> *mut XhciControllerImpl;
    fn UsbXhciController_initialize(c_impl: *mut XhciControllerImpl) -> i32;
    fn UsbXhciController_run(c_impl: *mut XhciControllerImpl) -> i32;
    fn UsbXhciController_configurePort(c_impl: *mut XhciControllerImpl);
    fn UsbXhciController_ProcessXhcEvent(c_impl: *mut XhciControllerImpl) -> i32;

    /// ref: https://doc.rust-lang.org/nomicon/ffi.html#targeting-callbacks-to-rust-objects
    fn RegisterMouseObserver(cb: extern "C" fn(displacement_x: i8, displacement_y: i8));
}

pub fn register_mouse_observer(cb: extern "C" fn(displacement_x: i8, displacement_y: i8)) {
    unsafe { RegisterMouseObserver(cb) };
}

enum XhciControllerImpl {}

pub struct XhciController {
    c_impl: *mut XhciControllerImpl,
}

impl XhciController {
    pub fn new(xhc_mmio_base: u64) -> Self {
        unsafe {
            Self {
                c_impl: UsbXhciController(xhc_mmio_base),
            }
        }
    }

    pub fn initialize(&self) -> Result<(), Error> {
        let error = unsafe { UsbXhciController_initialize(self.c_impl) };
        match convert_to_code(error) {
            None => Ok(()),
            Some(code) => Err(code),
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        let error = unsafe { UsbXhciController_run(self.c_impl) };
        println!("XhciController.run finished");
        match convert_to_code(error) {
            None => Ok(()),
            Some(code) => Err(code),
        }
    }

    pub fn configure_port(&self) {
        unsafe { UsbXhciController_configurePort(self.c_impl) };
        println!("XchiController.configure_port finished");
    }

    pub fn process_event(&self) -> Result<(), Error> {
        let error = unsafe { UsbXhciController_ProcessXhcEvent(self.c_impl) };
        //println!("XchiController.process_event finished. code = {}", error);
        match convert_to_code(error) {
            None => Ok(()),
            Some(code) => Err(code),
        }
    }
}

fn convert_to_code(code: i32) -> Option<Error> {
    if code == 0 {
        // Success
        return None;
    }

    println!("a cpp error occurs. code = {}", code);

    let code = match code {
        1 => Error::FULL,
        2 => Error::EMPTY,
        3 => Error::NO_ENOUGH_MEMORY,
        4 => Error::INDEX_OUT_OF_RANGE,
        5 => Error::HOST_CONTROLLER_NOT_HALTED,
        6 => Error::INVALID_SLOT_ID,
        7 => Error::PORT_NOT_CONNECTED,
        8 => Error::INVALID_ENDPOINT_NUMBER,
        9 => Error::TRANSFER_RING_NOT_SET,
        10 => Error::ALREADY_ALLOCATED,
        11 => Error::NOT_IMPLEMENTED,
        12 => Error::INVALID_DESCRIPTOR,
        13 => Error::BUFFER_TOO_SMALL,
        14 => Error::UNKNOWN_DEVICE,
        15 => Error::NO_CORRESPONDING_SETUP_STAGE,
        16 => Error::TRANSFER_FAILED,
        17 => Error::INVALID_PHASE,
        18 => Error::UNKNOWN_XHCI_SPEED_ID,
        19 => Error::NO_WAITER,
        20 => Error::LAST_OF_CODE,
        _ => {
            panic!("unexpected code {}", code);
        }
    };

    Some(code)
}
