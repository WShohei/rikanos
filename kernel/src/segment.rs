use core::ptr::addr_of_mut;
use core::mem::MaybeUninit;

//static mut GDT: [SegmentDescriptor; 3] = [SegmentDescriptor::new(); 3];
static mut GDT: MaybeUninit<[SegmentDescriptor; 3]> = MaybeUninit::uninit();

#[derive(Debug, Clone, Copy)]
struct DescriptorType(u64);

#[allow(dead_code)]
impl DescriptorType {
    pub const UPPER8BYTES: DescriptorType = DescriptorType(0);
    pub const LDT: DescriptorType = DescriptorType(2);
    pub const TSS_AVAILABLE: DescriptorType = DescriptorType(9);
    pub const TSS_BUSY: DescriptorType = DescriptorType(11);
    pub const CALL_GATE: DescriptorType = DescriptorType(12);
    pub const INTERRUPT_GATE: DescriptorType = DescriptorType(14);
    pub const TRAP_GATE: DescriptorType = DescriptorType(15);

    pub const READ_WRITE: DescriptorType = DescriptorType(2);
    pub const EXECUTE_READ: DescriptorType = DescriptorType(10);

    pub const UNDEFINED: DescriptorType = DescriptorType(4);
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentDescriptor {
    data: u64,
    bits: SegmentDescriptorBits,
}

#[allow(dead_code)]
impl SegmentDescriptor {
    pub const fn new() -> Self {
        SegmentDescriptor {
            data: 0,
            bits: SegmentDescriptorBits::new(),
        }
    }

    fn set_data(&mut self, data: u64) {
        self.data = data;
    }

    fn set_base(&mut self, base: u64) {
        self.bits.base_low = base & 0xffff;
        self.bits.base_middle = (base >> 16) & 0xff;
        self.bits.base_high = (base >> 24) & 0xff;
    }

    fn set_limit(&mut self, limit: u64) {
        self.bits.limit_low = limit & 0xffff;
        self.bits.limit_high = (limit >> 16) & 0xf;
    }

    fn set_type(&mut self, ty: DescriptorType) {
        self.bits.descriptor_type = ty;
    }

    fn set_present(&mut self, present: bool) {
        self.bits.present = if present { 1 } else { 0 };
    }

    fn set_privilege_level(&mut self, dpl: u64) {
        self.bits.descriptor_privilege_level = dpl & 0x3;
    }

    fn set_long_mode(&mut self, long_mode: bool) {
        self.bits.long_mode = if long_mode { 1 } else { 0 };
    }

    fn set_default_operation_size(&mut self, default_operation_size: bool) {
        self.bits.default_operation_size = if default_operation_size { 1 } else { 0 };
    }

    fn set_granularity(&mut self, granularity: bool) {
        self.bits.granularity = if granularity { 1 } else { 0 };
    }

    fn set_system_segment(&mut self, system_segment: bool) {
        self.bits.system_segment = if system_segment { 1 } else { 0 };
    }

    fn set_available(&mut self, available: bool) {
        self.bits.available = if available { 1 } else { 0 };
    }
}

#[derive(Debug, Clone, Copy)]
struct SegmentDescriptorBits {
    limit_low: u64,
    base_low: u64,
    base_middle: u64,
    descriptor_type: DescriptorType,
    system_segment: u64,
    descriptor_privilege_level: u64,
    present: u64,
    limit_high: u64,
    available: u64,
    long_mode: u64,
    default_operation_size: u64,
    granularity: u64,
    base_high: u64,
}

impl SegmentDescriptorBits {
    const fn new() -> Self {
        SegmentDescriptorBits {
            limit_low: 16,
            base_low: 16,
            base_middle: 8,
            descriptor_type: DescriptorType::UNDEFINED,
            system_segment: 1,
            descriptor_privilege_level: 2,
            present: 1,
            limit_high: 4,
            available: 1,
            long_mode: 1,
            default_operation_size: 1,
            granularity: 1,
            base_high: 8,
        }
    }
}

fn set_code_segment(
    desc: &mut SegmentDescriptor,
    desc_type: DescriptorType,
    dpl: u64,
    base: u64,
    limit: u64,
) {
    desc.set_base(base);
    desc.set_limit(limit);
    desc.set_type(desc_type);
    desc.set_present(true);
    desc.set_privilege_level(dpl);
    desc.set_long_mode(true);
    desc.set_default_operation_size(false);
    desc.set_granularity(true);
    desc.set_system_segment(true);
    desc.set_available(false);
}

fn set_data_segment(
    desc: &mut SegmentDescriptor,
    desc_type: DescriptorType,
    dpl: u64,
    base: u64,
    limit: u64,
) {
    set_code_segment(desc, desc_type, dpl, base, limit);
    desc.set_long_mode(false);
    desc.set_default_operation_size(true); // 32bit
}


extern "C" {
    fn load_gdt(limit: u16, offset: u64);
    pub fn set_dsall(value: u16);
    pub fn set_csss(cs: u16, ss: u16);
}

pub fn setup_segments() {
    unsafe {
        let gdt_ptr = &mut *GDT.as_mut_ptr();
        let gdt = addr_of_mut!(gdt_ptr[0]);
        let gdt = gdt as *mut SegmentDescriptor;
        let gdt = core::slice::from_raw_parts_mut(gdt, 3);
        let gdt_size = core::mem::size_of_val(gdt) as u16;
        gdt[0].set_data(0);
        set_code_segment(&mut gdt[1], DescriptorType::EXECUTE_READ, 0, 0, 0xfffff);
        set_data_segment(&mut gdt[2], DescriptorType::READ_WRITE, 0, 0, 0xfffff);
        load_gdt(gdt_size - 1, addr_of_mut!(GDT) as u64);
    }
}
