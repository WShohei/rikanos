use crate::error::{Error, Result};
use crate::{print, println};

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
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

fn write_address(address: u32) {
    unsafe {
        IoOut32(PCI_CONFIG_ADDRESS as u32, address);
    }
}

fn write_data(value: u32) {
    unsafe {
        IoOut32(PCI_CONFIG_DATA as u32, value);
    }
}

fn read_data() -> u32 {
    unsafe { IoIn32(PCI_CONFIG_DATA as u32) }
}

fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    let address = make_address(bus, device, function, 0);
    write_address(address);
    read_data() as u16
}

fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    let address = make_address(bus, device, function, 0);
    write_address(address);
    (read_data() >> 16) as u16
}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    let address = make_address(bus, device, function, 0xc);
    write_address(address);
    ((read_data() >> 16) & 0xff) as u8
}

fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
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

static mut DEVICES: [Option<Device>; 32] = [None; 32];
static mut NUM_DEVICES: usize = 0;
static mut IS_INITIALIZED: bool = false;

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
}

impl Device {
    pub fn initialize() -> Error {
        if unsafe { IS_INITIALIZED } {
            return Error::ALREADY_ALLOCATED;
        }
        unsafe {
            IS_INITIALIZED = true;
        }
        Device::scan_all_bus()
    }

    pub fn get_device(n: usize) -> Option<Device> {
        unsafe { DEVICES[n] }
    }

    pub fn num_devices() -> usize {
        unsafe { NUM_DEVICES }
    }

    pub fn find_xhc_device() -> Option<Device> {
        if !unsafe { IS_INITIALIZED } {
            println!("Device is not initialized");
            return None;
        }
        let num_devices = unsafe { NUM_DEVICES };
        for i in 0..num_devices {
            let device = unsafe { DEVICES[i] };
            if let Some(device) = device {
                let class_code = device.read_class_code();
                let base = ((class_code >> 24) & 0xff) as u8;
                let sub = ((class_code >> 16) & 0xff) as u8;
                let interface = ((class_code >> 8) & 0xff) as u8;
                if (base == 0x0c) && (sub == 0x03) && (interface == 0x30) {
                    return Some(device);
                }
            }
        }
        None
    }

    pub fn switch_ehci2xhci(&self) -> Result<()> {
        let class_code = self.read_class_code();
        let base = ((class_code >> 24) & 0xff) as u8;
        let sub = ((class_code >> 16) & 0xff) as u8;
        let interface = ((class_code >> 8) & 0xff) as u8;
        if (base == 0x0c)
            && (sub == 0x03)
            && (interface == 0x20)
            && (self.read_vendor_id()) == 0x8086
        {
            let superspeed_ports = self.read_reg(0xdc); // USB3PRM
            self.write_reg(0xd8, superspeed_ports); // USB3_PSSEN
            let ehci2xhci_ports = self.read_reg(0xd4); // USB2PRM
            self.write_reg(0xd0, ehci2xhci_ports); // XUSB2PR 
            println!("Switch EHCI to xHCI: SS = {:#x}, EHCI2xHCI = {:#x}", superspeed_ports, ehci2xhci_ports);
            return Ok(());
        } else {
            return Err(Error::UNKNOWN_DEVICE)
        }
    }

    fn scan_all_bus() -> Error {
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

    pub fn read_vendor_id(&self) -> u16 {
        read_vendor_id(self.bus, self.device, self.function)
    }

    pub fn read_device_id(&self) -> u16 {
        read_device_id(self.bus, self.device, self.function)
    }

    pub fn read_header_type(&self) -> u8 {
        read_header_type(self.bus, self.device, self.function)
    }

    pub fn read_class_code(&self) -> u32 {
        read_class_code(self.bus, self.device, self.function)
    }

    pub fn read_bus_numbers(&self) -> u32 {
        read_bus_numbers(self.bus, self.device, self.function)
    }

    fn read_reg(&self, reg_addr: u8) -> u32 {
        let address = make_address(self.bus, self.device, self.function, reg_addr);
        write_address(address);
        read_data()
    }

    fn write_reg(&self, reg_addr: u8, value: u32) {
        let address = make_address(self.bus, self.device, self.function, reg_addr);
        write_address(address);
        write_data(value);
    }

    pub fn read_bar(&self, bar: u8) -> Result<u64> {
        if bar >= 5 {
            println!("invalid bar number. bar must be less than 5");
            return Err(Error::INDEX_OUT_OF_RANGE);
        } else if bar % 4 != 0 {
            println!("invalid bar number. bar must be multiple of 4");
            return Err(Error::INDEX_OUT_OF_RANGE);
        }

        let offset1 = 0x10 + bar * 4;
        let bar1 = self.read_reg(offset1) as u64;

        let offset2 = offset1 + 4;
        let bar2 = self.read_reg(offset2) as u64;

        Ok((bar2 << 32) | bar1)
    }
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
