// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#![allow(unused_imports)]

use multiboot2::{
	FramebufferTag,
	InfoHeader,
	LoadBaseAddrTag,
	MemoryType,
	MmapTag,
	NewACPITag,
	Tag,
	TagBaseType,
	find_tag,
	tail_of,
};

use core::{
	hint::cold_path,
	mem::zeroed,
	ops::IndexMut,
	ptr::{
		NonNull,
		addr_of,
		addr_of_mut,
		null,
	},
	slice::{
		from_raw_parts,
		from_raw_parts_mut,
	},
};

use crate::{
	arch::x86_64::io::{
		hlt,
		outb,
	},
	bootparams::{
		BootMapEntry,
		BootMemMap,
		KERNEL_BOOT_INFO,
		KernelBootInfo,
		KernelBootType,
	},
	include::types::{
		PhysAddr,
		VirtAddr,
	},
	mm::buddy,
	pagetbl::{
		L2E_SIZE,
		L3E_SIZE,
		L4E_SIZE,
		PAGESIZE,
		PT_ENTRIES,
		PTE,
		PageFlags,
		PageFrame,
		PageLeaf,
		PageTable,
		l2_leaf,
		l3_leaf,
		l4_leaf,
		page_align_down,
		set_cr3,
	},
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


#[derive(Debug)]
enum EarlyInitError {
	OutOfMemory,
	TagError,
	PtrError,
	NoMemoryMap,
	NoFramebuffer,
	NoACPI,
	RegionCap,
	BuddyError,
}

type Result<T> = core::result::Result<T, EarlyInitError>;

const BOOTMEM_MAX_REGIONS: usize = 64;

struct BootRegion {
	current: usize,
	base: usize,

	cur_frames: usize,
	max_frames: usize,
}

struct BootAllocator {
	regions: [Option<NonNull<BootRegion>>; BOOTMEM_MAX_REGIONS],
	count: usize,
}


impl BootRegion {
	unsafe fn new(base: usize, len: usize) -> Option<&'static mut Self> {
		if !len.is_multiple_of(PAGESIZE) {
			return None;
		}
		let base = base & !(PAGESIZE - 1);

		let region = base as *mut BootRegion;
		unsafe {
			*region = BootRegion {
				current: base.saturating_add(PAGESIZE),
				base,
				cur_frames: (len >> 12) - 1,
				max_frames: len >> 12,
			};

			Some(region.as_mut_unchecked())
		}
	}

	fn alloc_frames(&mut self, count: usize) -> Option<*mut u8> {
		if count > self.cur_frames {
			return None;
		}

		let addr = self.current;
		self.current += count.saturating_mul(PAGESIZE);
		self.cur_frames -= count;

		let ptr = addr as *mut u8;
		unsafe {
			ptr.write_bytes(zeroed(), PAGESIZE);
		}

		Some(addr as *mut _)
	}

	fn alloc_frame(&mut self) -> Option<*mut u8> {
		if self.cur_frames == 0 {
			return None;
		}

		let addr = self.current;
		self.current += PAGESIZE;
		self.cur_frames -= 1;

		let ptr = addr as *mut u8;
		unsafe {
			ptr.write_bytes(zeroed(), PAGESIZE);
		}

		Some(addr as *mut _)
	}
}


impl BootAllocator {
	pub fn alloc(&self) -> Result<*mut u8> {
		unsafe {
			self.regions[0]
				.ok_or(EarlyInitError::OutOfMemory)?
				.as_mut()
				.alloc_frame()
				.ok_or(EarlyInitError::OutOfMemory)
		}
	}

	pub fn alloc_range_of_frames(&self, count: usize) -> Result<*mut u8> {
		unsafe {
			self.regions[0]
				.ok_or(EarlyInitError::OutOfMemory)?
				.as_mut()
				.alloc_frames(count)
				.ok_or(EarlyInitError::OutOfMemory)
		}
	}

	const fn new() -> Self {
		Self {
			regions: [const { None }; BOOTMEM_MAX_REGIONS],
			count: 0,
		}
	}

	fn add_region(&mut self, entry: BootMapEntry) -> Result<()> {
		if self.count >= BOOTMEM_MAX_REGIONS {
			return Err(EarlyInitError::RegionCap);
		}
		if self.regions[self.count].is_some() {
			return Err(EarlyInitError::RegionCap);
		}

		let new = unsafe {
			BootRegion::new(entry.base.as_usize(), entry.len).ok_or(EarlyInitError::PtrError)?
		};
		self.regions[self.count] = Some(NonNull::from(new));

		self.count += 1;
		Ok(())
	}

	// fn sort_regions(&mut self) {
	// 	unsafe { self.regions.sort_unstable_by_key(|o| o.map(|mut r| r.as_mut().base)) };
	// }

	fn add_region_entries(&mut self, usable_regions: &[BootMapEntry]) -> Result<()> {
		for region in usable_regions {
			if region.len > PAGESIZE {
				self.add_region(*region)?;
			}
		}

		Ok(())
	}

	pub fn pte_alloc<'a>(&self, pte: &'a mut PTE) -> Result<&'a mut PageTable> {
		unsafe {
			if pte.as_u64() & PageFlags::PRESENT.as_u64() == 0 {
				let table = self.alloc()?.cast::<PageTable>();
				pte.link(table, PageFlags::READ_WRITE);
				Ok(table.as_mut_unchecked())
			} else {
				Ok(pte.rebase().as_mut_unchecked())
			}
		}
	}
}


fn parse_mapentries(map: &MmapTag, load_addr: usize) -> BootMemMap {
	let mut bootmap: BootMemMap = unsafe { zeroed() };
	let maps = unsafe { tail_of::<MmapTag>(&map.tag as *const _) };

	let mut j = 0;
	let mut i = 0;

	let kernel_start = addr_of!(_kphys_start) as usize;
	let kernel_end = addr_of!(_kphys_end) as usize;
	let kernel_size = (kernel_end - kernel_start) + load_addr;

	for ent in maps {
		let (base, len) = if ent.addr == 0 {
			if ent.len as usize <= PAGESIZE {
				bootmap.ram_length += ent.len as usize;
				continue;
			} else {
				(ent.addr as usize + PAGESIZE, ent.len as usize - PAGESIZE)
			}
		} else if ent.addr as usize >= kernel_start && ((ent.addr + ent.len) as usize) < kernel_end
		{
			(ent.addr as usize + kernel_size, ent.len as usize - kernel_size)
		} else {
			(ent.addr as usize, ent.len as usize)
		};

		bootmap.ram_length += ent.len as usize;

		if ent.r#type == MemoryType::ACPI_RECLAIMABLE as u32 {
			core::hint::cold_path();
			if j >= bootmap.entries_recl.len() {
				continue;
			}

			bootmap.entries_recl[j].base = PhysAddr(base);
			bootmap.entries_recl[j].len = len;
			bootmap.ram_usable += len;
			bootmap.count_recl += 1;
			j += 1;
		} else if ent.r#type == MemoryType::AVAILABLE as u32 {
			if i >= bootmap.entries.len() {
				continue;
			}

			bootmap.entries[i].base = PhysAddr(base);
			bootmap.entries[i].len = len;
			bootmap.ram_usable += len;
			bootmap.count += 1;
			i += 1;
		}
	}

	bootmap.entries.sort_unstable_by_key(|p| p.base);
	bootmap.entries_recl.sort_unstable_by_key(|p| p.base);

	bootmap
}


#[rustfmt::skip]
fn parse_mb2(info: *const InfoHeader) -> Option<KernelBootInfo> {
    let mut boot_info: KernelBootInfo = unsafe { zeroed() };

    let map;
    let fb;
    let _acpi;

    unsafe {
        map   = find_tag(info, TagBaseType::MMAP)?.cast::<MmapTag>().as_ref_unchecked();
        fb    = find_tag(info, TagBaseType::FRAMEBUFFER)?.cast::<FramebufferTag>().as_ref_unchecked();
        _acpi = find_tag(info, TagBaseType::ACPI_NEW)?.cast::<NewACPITag>().as_ref_unchecked();
        // TODO add acpi

        boot_info.load_addr = match find_tag(info, TagBaseType::LOAD_BASE_ADDR) {
        Some(s) => PhysAddr((*s.cast::<LoadBaseAddrTag>()).load_base_addr as usize),
        None => PhysAddr(1_usize << 20_usize), // 1 << 20 is the guess of where we're loaded
        };

        boot_info.maps = parse_mapentries(map, boot_info.load_addr.as_usize());

        boot_info.bootenv = if (find_tag(info, TagBaseType::EFI64)).is_some() {
            KernelBootType::EFI64
        } else if (find_tag(info, TagBaseType::EFI32)).is_some() {
            KernelBootType::EFI32
        } else {
            cold_path();
            KernelBootType::BIOS
        };
    }

    boot_info.fb.virt = VirtAddr(0);
    boot_info.fb.phys = PhysAddr(fb.addr as usize);
    boot_info.fb.width = fb.width;
    boot_info.fb.height = fb.height;
    boot_info.fb.pitch = fb.pitch;
    boot_info.fb.bpp = fb.bpp;

    Some(boot_info)
}


const KB: usize = 1024;
const MB: usize = KB * 1024;
const GB: usize = MB * 1024;

const MAX_ENT: usize = PT_ENTRIES;


fn setup_kernmap(
	bootmem: &mut BootAllocator,
	l4: &mut PageTable,
	boot_info: &KernelBootInfo,
) -> Result<()> {
	let mut vaddr = addr_of!(KERNEL_OFFSET) as usize;

	let kernel_start = addr_of!(_kphys_start) as usize;
	let kernel_end = addr_of!(_kphys_end) as usize;
	let kernel_size = (kernel_end - kernel_start) + boot_info.load_addr.as_usize();

	let count_l3 = kernel_size.div_ceil(L4E_SIZE);
	let count_l2 = kernel_size.div_ceil(L3E_SIZE);
	let count_l1 = kernel_size.div_ceil(L2E_SIZE);

	let mut paddr = 0x0;
	let mut l2_remaining = count_l2;
	let mut l1_remaining = count_l1;

	for _ in 0..count_l3 {
		let l4e = l4.index_leaf(l4_leaf(vaddr));
		let l3 = bootmem.pte_alloc(l4e)?;

		let l2_count = if l2_remaining > MAX_ENT { MAX_ENT } else { l2_remaining };

		for _ in 0..l2_count {
			let l3e = l3.index_leaf(l3_leaf(vaddr));
			let l2 = bootmem.pte_alloc(l3e)?;

			let l1_count = if l1_remaining > MAX_ENT { MAX_ENT } else { l1_remaining };

			for _ in 0..l1_count {
				let l2e = l2.index_leaf(l2_leaf(vaddr));
				let l1 = bootmem.pte_alloc(l2e)?;

				for pe in &mut l1.0 {
					unsafe { pe.link(paddr as *const _, PageFlags::READ_WRITE) };

					paddr += KB * 4;
					vaddr += KB * 4;
				}
			}

			l1_remaining -= l1_count;
		}

		l2_remaining -= l2_count;
	}

	Ok(())
}

fn setup_physmap(bootmem: &mut BootAllocator, l4: &mut PageTable, map: &BootMemMap) -> Result<()> {
	let mut vaddr = addr_of!(KERNEL_PHMMAP) as usize;
	let ram_size = map.ram_length;

	let count_l3 = ram_size.div_ceil(L4E_SIZE);
	let count_l2 = ram_size.div_ceil(L3E_SIZE);
	let count_l1 = ram_size.div_ceil(L2E_SIZE);

	let mut paddr = 0x0;
	let mut l2_remaining = count_l2;
	let mut l1_remaining = count_l1;

	for _ in 0..count_l3 {
		let l4e = l4.index_leaf(l4_leaf(vaddr));
		let l3 = bootmem.pte_alloc(l4e)?;

		let l2_count = if l2_remaining > MAX_ENT { MAX_ENT } else { l2_remaining };

		for _ in 0..l2_count {
			let l3e = l3.index_leaf(l3_leaf(vaddr));
			let l2 = bootmem.pte_alloc(l3e)?;

			let l1_count = if l1_remaining > MAX_ENT { MAX_ENT } else { l1_remaining };

			for _ in 0..l1_count {
				let l2e = l2.index_leaf(l2_leaf(vaddr));
				unsafe {
					l2e.link(paddr as *const _, PageFlags::HUGE);
				}

				vaddr += MB * 2;
				paddr += MB * 2;
			}

			l1_remaining -= l1_count;
		}

		l2_remaining -= l2_count;
	}

	Ok(())
}

fn setup_identmap(bootmem: &mut BootAllocator, l4: &mut PageTable) -> Result<()> {
	let l4e = l4.index_leaf(PageLeaf::from(0));
	let l3 = bootmem.pte_alloc(l4e)?;
	let l3e = l3.index_leaf(PageLeaf::from(0));
	let l2 = bootmem.pte_alloc(l3e)?;

	const identmap_size: usize = GB;
	const count_l1: usize = identmap_size.div_ceil(L2E_SIZE);

	let mut phys = 0x0;
	let mut l2e;

	for _ in 0..PT_ENTRIES {
		l2e = l2.index_leaf(l2_leaf(phys));
		unsafe {
			l2e.link(phys as *const _, PageFlags::HUGE);
		}

		phys += MB * 2;
	}

	Ok(())
}


fn setup_buddy(bootmem: &mut BootAllocator, bootinfo: &KernelBootInfo) -> Result<()> {
	let mut master: buddy::Master = buddy::Master::new();

	master.ram_length = bootinfo.maps.ram_length;
	master.usable_ram = bootinfo.maps.ram_usable;

	buddy::MASTER_BUDDY.set(master).map_err(|_| EarlyInitError::BuddyError)?;

	unsafe {
		for region in bootmem.regions.iter().skip(1) {
			if let Some(refregion) = region.map(|p| p.as_ref()) {
				let base = PhysAddr(refregion.base).to_phmap().as_mut_ptr::<u8>();
				let len = refregion.max_frames << 12;

				base.cast::<u8>().write_bytes(zeroed(), size_of::<buddy::Slave>());

				buddy::buddy_free_range(base, len).map_err(|_| EarlyInitError::BuddyError)?;
			}
		}
	}

	Ok(())
}


fn init_kernel_env(bootmem: &mut BootAllocator, info: &KernelBootInfo) -> Result<()> {
	let l4 =
		unsafe { bootmem.alloc()?.cast::<PageTable>().as_mut().ok_or(EarlyInitError::PtrError)? };
	// we use 2mb large pages because we don't know if the CPU supports 1GB hugepages yet

	setup_kernmap(bootmem, l4, info)?;
	setup_physmap(bootmem, l4, &info.maps)?;
	setup_identmap(bootmem, l4)?;
	unsafe { set_cr3(PhysAddr(l4 as *const _ as usize)) };
	setup_buddy(bootmem, info)?;

	Ok(())
}


fn gather_boot_info(info: *const InfoHeader) -> Result<&'static KernelBootInfo> {
	KERNEL_BOOT_INFO
		.set(parse_mb2(info).ok_or(EarlyInitError::TagError)?)
		.map_err(|_| EarlyInitError::TagError)?;
	KERNEL_BOOT_INFO.get().ok_or(EarlyInitError::PtrError)
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_kernel(info: *const InfoHeader) -> ! {
	let boot_info = match gather_boot_info(info) {
	Ok(v) => v,
	Err(_) => die("early boot info gathering failure!"),
	};

	let mut bootmem = BootAllocator::new();
	let _ = bootmem.add_region_entries(&boot_info.maps.entries);
	init_kernel_env(&mut bootmem, boot_info).unwrap_or_else(|_| die("early init boot failure!"));

	boot_trampoline();
}

#[cold]
fn die(str: &'static str) -> ! {
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

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn boot_trampoline() -> ! {
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

// hi
