use crate::error::Error;

pub fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let shl = |x: u32, bits: usize| x << bits;

    shl(1 as u32, 31)
        | shl(bus as u32, 16)
        | shl(device as u32, 11)
        | shl(function as u32, 8)
        | (reg_addr & 0xfc) as u32
}

static PCI_CONFIG_ADDRESS: u16 = 0xcf8;
static PCI_CONFIG_DATA: u16 = 0xcfc;

extern "C" {
    fn IoOut32(addr: u32, data: u32) -> ();
    fn IoIn32(addr: u32) -> u32;
}

pub fn write_address(address: u32) {
    unsafe {
        IoOut32(PCI_CONFIG_ADDRESS as u32, address);
    }
}

pub fn write_data(value: u32) {
    unsafe {
        IoOut32(PCI_CONFIG_DATA as u32, value);
    }
}

pub fn read_data() -> u32 {
    unsafe { IoIn32(PCI_CONFIG_DATA as u32) }
}

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    let address = make_address(bus, device, function, 0);
    write_address(address);
    read_data() as u16
}

//fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
//    let address = make_address(bus, device, function, 0);
//    write_address(address);
//    (read_data() >> 16) as u16
//}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    let address = make_address(bus, device, function, 0xc);
    write_address(address);
    ((read_data() >> 16) & 0xff) as u8
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    let address = make_address(bus, device, function, 0x8);
    write_address(address);
    read_data()
}

fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    let address = make_address(bus, device, function, 0x18);
    write_address(address);
    read_data()
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
}

pub static mut DEVICES: [Option<Device>; 32] = [None; 32];
pub static mut NUM_DEVICES: usize = 0;

pub fn get_device(n: usize) -> Option<Device> {
    unsafe { DEVICES[n] }
}

pub fn scan_all_bus() -> Error {
    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }

    for function in 0..8 {
        if read_vendor_id(0, 0, function) != 0xffff {
            let e = scan_bus(function);
            if e != Error::SUCCESS {
                return e;
            }
        }
    }

    Error::SUCCESS
}

fn scan_bus(bus: u8) -> Error {
    for device in 0..32 {
        if read_vendor_id(bus, device, 0) != 0xffff {
            let e = scan_device(bus, device);
            if e != Error::SUCCESS {
                return e;
            }
        }
    }
    Error::SUCCESS
}

fn scan_device(bus: u8, device: u8) -> Error {
    let e = scan_function(bus, device, 0);
    if e != Error::SUCCESS {
        return e;
    }
    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Error::SUCCESS;
    }

    for function in 1..8 {
        if read_vendor_id(bus, device, function) != 0xffff {
            let e = scan_function(bus, device, function);
            if e != Error::SUCCESS {
                return e;
            }
        }
    }

    Error::SUCCESS
}

fn scan_function(bus: u8, device: u8, function: u8) -> Error {
    let header_type = read_header_type(bus, device, function);
    let e = add_device(bus, device, function, header_type);
    if e != Error::SUCCESS {
        return e;
    }

    let class_code = read_class_code(bus, device, function);
    let base = ((class_code >> 24) & 0xff) as u8;
    let sub = ((class_code >> 16) & 0xff) as u8;

    if (base == 0x06) && (sub == 0x04) {
        // standard PCI-PCI bridge
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = ((bus_numbers >> 8) & 0xff) as u8;
        return scan_bus(secondary_bus);
    }

    return Error::SUCCESS;
}

fn add_device(bus: u8, device: u8, function: u8, header_type: u8) -> Error {
    if unsafe { NUM_DEVICES } >= unsafe { DEVICES.len() } {
        return Error::FULL;
    }

    unsafe {
        DEVICES[NUM_DEVICES] = Some(Device {
            bus,
            device,
            function,
            header_type,
        });
        NUM_DEVICES += 1;
    }

    Error::SUCCESS
}
