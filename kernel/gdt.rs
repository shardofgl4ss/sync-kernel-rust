use core::mem::offset_of;
use core::ptr::addr_of;

type quad = u64;
type long = u32;
type word = u16;
type byte = u8;

mod GDT_Bits {
    /// loaded in memory.
    const PRESENT: u8 = 1 << 7;
    /// DPL 0: kernelspace
    const DPL_RING0: u8 = 0;
    /// DPL 3: userspace
    const DPL_RING3: u8 = 1 << 5 | 1 << 6;
    /// set for code/data, unset for tss/ldt gates/etc
    const DESC_S: u8 = 1 << 4;
    /// used by cpu
    const ACCESSED: u8 = 1 << 0;

    mod ds {
        pub const DIRECTION: u8 = 1 << 2;
        pub const WRITABLE: u8 = 1 << 1;
    }

    mod cs {
        pub const EXECUTABLE: u8 = 1 << 3;
        pub const CONFORMING: u8 = 1 << 2;
        pub const READABLE: u8 = 1 << 1;
    }

    mod flags {
        pub const GRANULARITY: u8 = 1 << 3;
        pub const PROT_MODE: u8 = 1 << 2;
        pub const LONG_MODE: u8 = 1 << 1;
        pub const AVL: u8 = 1 << 0;
    }

    pub const KERNEL_CS_PPT: u8 = PRESENT | DPL_RING0 | DESC_S | cs::EXECUTABLE | cs::READABLE;
    pub const KERNEL_DS_PPT: u8 = PRESENT | DPL_RING0 | DESC_S | ds::WRITABLE;
    pub const KERNEL_CS_FLAGS: u8 = flags::LONG_MODE << 4;
}


#[derive(Clone, Copy)]
#[repr(C, packed)]
struct GDTEntry {
    lo_limit: word,

    zero: word,
    zero1: byte,

    ppt: byte,
    hi_limit_flags: byte,
    zero2: byte,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct GDT64 {
    null: quad,

    cs: GDTEntry,
}

#[repr(C, packed)]
struct GDTDescriptor {
    limit: word,
    base: *const GDT64,
}


static KGDT: GDT64 = GDT64 {
    null: 0,

    cs: GDTEntry {
        lo_limit: 0,
        zero: 0,
        zero1: 0,
        ppt: GDT_Bits::KERNEL_CS_PPT,
        hi_limit_flags: GDT_Bits::KERNEL_CS_FLAGS,
        zero2: 0,
    }
};

static GDTR: GDTDescriptor = GDTDescriptor {
    limit: (size_of::<GDT64>() - 1) as u16,
    base: addr_of!(KGDT),
};

unsafe impl Sync for GDTDescriptor {}


#[unsafe(no_mangle)]
pub static CODE_SEGMENT64: usize = offset_of!(GDT64, cs);


unsafe extern "C" {
     fn kern_gdt_init(gdt: *const GDTDescriptor);
}


pub fn init() {
    unsafe {
        kern_gdt_init(addr_of!(GDTR));
    }
}





