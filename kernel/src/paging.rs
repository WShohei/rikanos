use core::ptr::addr_of;

static PAGE_SIZE_4K: u64 = 4096;
static PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
static PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

static mut PML4_TABLE: TABLE = TABLE {
    table: [0; 512],
};
static mut PDP_TABLE: TABLE = TABLE {
    table: [0; 512],
};
static mut PAGE_DIR: DIR = DIR {
    dir: [[0; 512]; 64],
};

#[repr(align(4096))]
struct TABLE {
    table: [u64; 512],
}

#[repr(align(4096))]
struct DIR {
    dir: [[u64; 512]; 64],
}

extern "C" {
    fn set_cr3(addr: u64);
}

pub fn setup_identity_page_table() {
    unsafe {
        PML4_TABLE.table[0] = addr_of!(PDP_TABLE.table[0]).cast::<u64>() as u64 | 0x3;
        for i_pdpt in 0..64 {
            PDP_TABLE.table[i_pdpt] = addr_of!(PAGE_DIR.dir[i_pdpt]).cast::<u64>() as u64 | 0x3;
            for i_pd in 0..512 {
                PAGE_DIR.dir[i_pdpt][i_pd] = (i_pdpt as u64 * PAGE_SIZE_1G + i_pd as u64 * PAGE_SIZE_2M) | 0x83;
            }
        }
        set_cr3(addr_of!(PML4_TABLE.table[0]).cast::<u64>() as u64);
    }
}
