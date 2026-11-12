// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#[path = "multiboot2.rs"]
mod multiboot2;

use core::{
    hint::cold_path, mem::zeroed, ops::IndexMut, ptr::{
        addr_of,
        null,
    }, slice::{from_raw_parts, from_raw_parts_mut},
};

use crate::{
    arch::x86_64::io::{
        hlt,
        outb,
    },

    bootparams::{
        BootMapEntry,
        BootMemMap,
        KernelBootInfo,
        KernelBootType,
        KERNEL_BOOT_INFO,
    },
    include::types::{
        PhysAddr,
        VirtAddr,
    },

    init::multiboot2::{
        FramebufferTag,
        MemoryType,
        MmapTag,
        NewACPITag,
        find_tag,
        tail_of,
    },

    pagetbl::{
        L2E_SIZE,
        L3E_SIZE,
        L4E_SIZE,
        PAGESIZE,
        PTE,
        PageFlags,
        PageFrame,
        PageTable,
        l2_leaf,
        l3_leaf,
        l4_leaf,
        page_align_down,
        pt_rebase,
        set_cr3,
    },

    mm::buddy,
};

use multiboot2::{
    InfoHeader,
    LoadBaseAddrTag,
    Tag,
    TagBaseType,
};

unsafe extern "C" {
    pub unsafe fn kernel_main() -> !;
    static kstack_top: u8;
    static _kphys_start: u8;
    static _kphys_end: u8;
    static _begin: u8;
    static _after_ro: u8;
    static KERNEL_OFFSET: u8;
    static KERNEL_PHMMAP: u8;
}

const BOOTMEM_MAX_REGIONS: usize = 64;

struct BootMemRegion {
    top: *mut PageFrame,
    base: *mut u8,

    cur_frames: usize,
    max_frames: usize,
}

struct BootMemAllocator {
    regions: [*mut BootMemRegion; BOOTMEM_MAX_REGIONS],
    region_count: usize,
}

static mut BOOTALLOC: BootMemAllocator = unsafe { zeroed() };

#[cold]
unsafe fn die(str: &'static str) -> ! {
    const ISA_DEBUGCON: u16 = 0xE9;

    unsafe {
        for ch in str.as_bytes() {
            outb(ISA_DEBUGCON, *ch);
        }

        outb(ISA_DEBUGCON, b'\n');

        loop {
            hlt();
        }
    }
}

impl BootMemAllocator {
    fn alloc_add_region(&mut self, entry: &mut BootMapEntry, idx: usize) {
        let start = entry.base.as_usize();
        let end = start + entry.len;
        let kernel_size = addr_of!(_kphys_end) as usize;

        let kstart = addr_of!(_kphys_start) as usize;
        let kend = addr_of!(_kphys_end) as usize;

        let mut addr = entry.base.as_usize() as *mut u8;
        let mut len = entry.len;

        if start <= kstart && end >= kend {
            cold_path();
            addr = unsafe { addr.add(len) };
            len -= kernel_size;
        }

        self.regions[idx] = addr as *mut BootMemRegion;
        let r = unsafe { self.regions[idx].as_mut_unchecked() };

        r.base = addr;
        r.max_frames = page_align_down(len) / PAGESIZE;

        r.cur_frames = r.max_frames - 1;
        r.top = unsafe { (r.base.byte_add(len) as *mut PageFrame).sub(1) };

        self.region_count += 1;
    }

    fn alloc_init(&mut self, maps: &mut BootMemMap) {
        if maps.count == 0 {
            cold_path();
            unsafe { die("no regions to use!") }
        };

        let mut regions: usize = 0;
        for i in 0..maps.count as usize {
            self.alloc_add_region(maps.entries.index_mut(i), i);
            regions += 1;
            if regions >= 64 {
                cold_path();
                // temporary
                unsafe { die("too many regions!") };
            }
        }
    }

    pub fn alloc_frame(&mut self) -> Option<*mut PageFrame> {
        let regions = unsafe { from_raw_parts_mut::<BootMemRegion>(self.regions[0], self.region_count) };

        for r in regions {
            if r.cur_frames == 0 {
                continue;
            }
            let frame = r.top;

            r.top = unsafe { r.top.sub(1) };
            r.cur_frames -= 1;

            unsafe { frame.write_bytes(0, PAGESIZE) };

            return Some(frame);
        }

        None
    }

    pub fn alloc_if_not_pt<'a>(&mut self, pte: &'a mut PTE, flags: PageFlags) -> &'a mut PageTable {
        if pte.as_u64() & PageFlags::PRESENT.as_u64() == 0 {
            unsafe {
                let page = self.alloc_frame().unwrap_or_else(|| die("page alloc map failure!"));
                pte.link(page as *const _ as *const PageTable, flags);
                &mut *(page as *mut PageTable)
            }
        } else {
            unsafe { &mut *(pt_rebase(*pte)) }
        }
    }
}




fn init_identity_map(alloc: &mut BootMemAllocator, l4: &mut PageTable) {
    let l4e = l4.index_leaf(0);
    let l3 = alloc.alloc_if_not_pt(l4e, PageFlags::READ_WRITE);

    let l3e = l3.index_leaf(0);
    unsafe { l3e.link(null::<PageTable>(), PageFlags::HUGE) };
}




fn init_kcode_map(alloc: &mut BootMemAllocator, l4: &mut PageTable, boot_info: &KernelBootInfo) {
    let mut phys: PhysAddr = PhysAddr(0x0);
    let k_idx = addr_of!(KERNEL_OFFSET) as usize;

    let k_start = addr_of!(_kphys_start) as usize;
    let k_end = addr_of!(_kphys_end) as usize;

    let kernel_size = (k_end - k_start) + boot_info.load_addr.as_usize();

    let count_l2 = kernel_size.div_ceil(L3E_SIZE);
    let count_l1 = kernel_size.div_ceil(L2E_SIZE);

    let l4_idx = l4_leaf(k_idx);
    let l3 = alloc.alloc_if_not_pt(l4.index_leaf(l4_idx), PageFlags::READ_WRITE);

    let robegin = VirtAddr(addr_of!(_begin) as usize);
    let roend = VirtAddr(addr_of!(_after_ro) as usize);


    for i in 0..count_l2 {
        let l3_idx = l3_leaf(k_idx + i * L3E_SIZE);
        let l2 = alloc.alloc_if_not_pt(l3.index_leaf(l3_idx), PageFlags::READ_WRITE);

        for j in 0..count_l1 {
            let l2_idx = l2_leaf(k_idx + j * L2E_SIZE);
            let l1 = alloc.alloc_if_not_pt(l2.index_leaf(l2_idx), PageFlags::READ_WRITE);

            for e in l1.0.iter_mut() {
                let mut flags = PageFlags::PRESENT;
                let va = phys.to_kcode();

                if va < robegin || va >= roend {
                    flags |= PageFlags::WRITABLE;
                }

                unsafe { (*e).link(phys.as_usize() as *const PageTable, flags) };
                phys = PhysAddr(phys.as_usize() + PAGESIZE);
            }
        }
    }
}




fn init_dram_map(alloc: &mut BootMemAllocator, l4: &mut PageTable, maps: &BootMemMap) {
    let mut voffs = addr_of!(KERNEL_PHMMAP) as usize;
    let ram_size = maps.ram_max; // ram max is to include the holes in memory

    let count_l3 = ram_size.div_ceil(L4E_SIZE);
    let count_l2 = ram_size.div_ceil(L3E_SIZE);

    let mut poffs = 0x0usize;
    let mut remaining = count_l2;

    for _ in 0..count_l3 {
        let l4_idx = l4_leaf(voffs);
        let l3 = alloc.alloc_if_not_pt(l4.index_leaf(l4_idx), PageFlags::READ_WRITE);

        let count = if remaining > 512 { 512 } else { remaining };

        for _ in 0..count {
            let l3_idx = l3_leaf(voffs);
            let l3e = l3.index_leaf(l3_idx);

            unsafe { l3e.link(poffs as *const PageTable, PageFlags::HUGE) };

            voffs += L3E_SIZE;
            poffs += L3E_SIZE;
        }

        remaining -= count;
    }
}




fn init_higher_half(alloc: &mut BootMemAllocator, boot_info: &KernelBootInfo) {
    let pml4 = unsafe { 
        match alloc.alloc_frame() {
        Some(v) => &mut *(v as *mut PageTable),
        None => die("OOM"),
        }
    };

    init_identity_map(alloc, pml4);
    init_kcode_map(alloc, pml4, boot_info);
    unsafe { set_cr3(PhysAddr(pml4 as *const _ as usize)) };
    init_dram_map(alloc, pml4, &boot_info.maps);
}




#[unsafe(naked)]
extern "C" fn trampoline() -> ! {
    core::arch::naked_asm!(
       ".cfi_startproc",
       "movabs $kstack_top, %rsp",
        "andq   $-0x10, %rsp",
        "pushq  $0",
        "jmp    kernel_main",
        ".cfi_endproc",
       options(att_syntax)
   );
}




// fn sort_bootmap_entries(maps: &mut [BootMapEntry], upto: usize) {
fn sort_bootmap_entries(maps: &mut [BootMapEntry], upto: usize) {
    for i in 1..upto {
        let k = maps[i];
        let mut j = i;

        while j > 0 && maps[j - 1].base > k.base {
            maps[j] = maps[j - 1];
            j -= 1;
        }

        maps[j] = k;
    }
}




fn parse_mapentries(boot_info: &mut KernelBootInfo, map: &MmapTag) {
    let bootmap = &mut boot_info.maps;
    let maps = unsafe { tail_of::<MmapTag>(&map.tag as *const Tag) };

    let mut j: usize = 0;
    let mut i: usize = 0;

    for ent in maps {
        if ent.r#type == MemoryType::ACPI_RECLAIMABLE as u32 {
            core::hint::cold_path();
            if j > bootmap.entries_recl.len() { continue; }

            bootmap.entries_recl[j].base = PhysAddr(ent.addr as usize);
            bootmap.entries_recl[j].len = ent.len as usize;
            bootmap.ram_max += ent.len as usize;
            bootmap.count_recl += 1;
            j += 1;
        } else if ent.r#type == MemoryType::AVAILABLE as u32 {
            if i > bootmap.entries.len() { continue; }

            bootmap.entries[i].base = PhysAddr(ent.addr as usize);
            bootmap.entries[i].len = ent.len as usize;
            bootmap.ram_max += ent.len as usize;
            bootmap.ram_usable += ent.len as usize;
            bootmap.count += 1;
            i += 1;
        }
    }

    sort_bootmap_entries(&mut bootmap.entries, bootmap.count as usize);
    sort_bootmap_entries(&mut bootmap.entries_recl, bootmap.count_recl as usize);
}

fn parse_mb2(info: *const InfoHeader, boot_info: &mut KernelBootInfo) {
    let map = unsafe { find_tag(info, TagBaseType::MMAP)
        .unwrap_or_else(|| die("no map!")) } as *const MmapTag;
    let fb = unsafe { find_tag(info, TagBaseType::FRAMEBUFFER)
        .unwrap_or_else(|| die("no framebuffer!")) } as *const FramebufferTag;
    // TOOD add acpi
    let _acpi = unsafe { find_tag(info, TagBaseType::ACPI_NEW)
        .unwrap_or_else(|| die("no acpi!")) } as *const NewACPITag;

    unsafe { parse_mapentries(boot_info, &(*map)) };

    boot_info.load_addr = unsafe {
        match find_tag(info, TagBaseType::LOAD_BASE_ADDR) {
        Some(s) => PhysAddr((*(s as *const LoadBaseAddrTag)).load_base_addr as usize),
        None => PhysAddr(1_usize<< 20_usize), // 1 << 20 is the guess of where we're loaded
        }
    };

    boot_info.bootenv = if (find_tag(info, TagBaseType::EFI64)).is_some() {
        KernelBootType::EFI64
    } else if (find_tag(info, TagBaseType::EFI32)).is_some() {
        KernelBootType::EFI32
    } else {
        cold_path();
        KernelBootType::BIOS
    };

    let bootfb = &mut boot_info.fb;
    let srcfb = unsafe { &(*fb) };

    bootfb.virt = VirtAddr(0);
    bootfb.phys = PhysAddr(srcfb.addr as usize);
    bootfb.width = srcfb.width;
    bootfb.height = srcfb.height;
    bootfb.pitch = srcfb.pitch;
    bootfb.bpp = srcfb.bpp;
}

fn buddy_handoff(boot_info: &KernelBootInfo, bootalloc: &BootMemAllocator) {
    let master = &raw mut buddy::MASTER_BUDDY;
    let master = unsafe { &mut (*master) };

    master.total_ram = boot_info.maps.ram_max;
    master.usable_ram = boot_info.maps.ram_usable;

    let regions = unsafe { from_raw_parts(addr_of!(*bootalloc.regions[0]), bootalloc.region_count) };

    for (i, r) in regions.iter().enumerate() {
        if i == 0 { continue }; // boot region will be remapped later
                                // also TODO: make region 0 the only used region

        let newbase = PhysAddr(r.base as usize).to_phmap().as_mut_ptr::<u8>();
        let len = r.cur_frames << 12;

        unsafe { newbase.write_bytes(0, size_of::<buddy::Slave>()); }
        let _ = buddy::buddy_free_range(newbase, len);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_kernel(info: *mut InfoHeader) -> ! {
    let mut boot_info: KernelBootInfo = unsafe { zeroed() };

    let alloc = &raw mut BOOTALLOC;
    let alloc = unsafe { &mut *(alloc) };

    parse_mb2(info, &mut boot_info);
    alloc.alloc_init(&mut boot_info.maps);
    init_higher_half(alloc, &boot_info);
    buddy_handoff(&boot_info, alloc);

    unsafe { KERNEL_BOOT_INFO = boot_info };

    trampoline();
}
