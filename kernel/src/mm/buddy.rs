// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::{
	cell::Cell,
	hint::cold_path,
	mem::zeroed,
	ptr::{
		NonNull,
		null_mut,
	},
	slice::from_raw_parts_mut,
	sync::atomic::{
		AtomicUsize,
		Ordering,
	},
};

use crate::{
	include::types::{
		PhysAddr,
		VirtAddr,
	},
	pagetbl::{
		PAGESIZE,
		PageFrame,
		page_align_down,
	},
	panic::kernel_panic,
	sync::{
		KGuard,
		KMutex,
		KOnce,
	},
};

unsafe extern "C" {
	static _kphys_start: u8;
	static _kphys_end: u8;
}

const BUDDY_MAX_ORDER: usize = 11;
const BUDDY_MAX_REGIONS: usize = 128;

type pfn = u64;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Order {
	ORDER_4K      = 0,
	ORDER_8K      = 1,
	ORDER_16K     = 2,
	ORDER_32K     = 3,
	ORDER_64K     = 4,
	ORDER_128K    = 5,
	ORDER_256K    = 6,
	ORDER_512K    = 7,
	ORDER_1024K   = 8,
	ORDER_2048K   = 9,
	ORDER_4096K   = 10,
	#[default]
	ORDER_INVALID = 1000,
}

impl Order {
	pub const fn val(self) -> usize {
		let order = self as usize;

		if order > BUDDY_MAX_ORDER - 1 { 0 } else { PAGESIZE << order }
	}
}

impl TryFrom<usize> for Order {
	type Error = usize;
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
		0 => Ok(Order::ORDER_4K),
		1 => Ok(Order::ORDER_8K),
		2 => Ok(Order::ORDER_16K),
		3 => Ok(Order::ORDER_32K),
		4 => Ok(Order::ORDER_64K),
		5 => Ok(Order::ORDER_128K),
		6 => Ok(Order::ORDER_256K),
		7 => Ok(Order::ORDER_512K),
		8 => Ok(Order::ORDER_1024K),
		9 => Ok(Order::ORDER_2048K),
		10 => Ok(Order::ORDER_4096K),
		_ => Err(value),
		}
	}
}

pub enum BuddyErr {
	InvalidBit,
	InvalidRegions,
	InvalidBitmap,
	InvalidParameters,
	NoBlock,
	NoBuddy,
	BadPtr,
	BadRegion,
	Misaligned,
	NotInList,
	MaxRegions,
	AlreadyExists,
	NotReady,
}

use super::buddy::BuddyErr::{
	AlreadyExists,
	BadRegion,
	InvalidBit,
	InvalidBitmap,
	InvalidParameters,
	InvalidRegions,
	Misaligned,
	NoBlock,
	NoBuddy,
	NotInList,
	NotReady,
};


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageFrameNumber(pub usize);

struct Bitmap {
	/// data :thumbsup:
	data: Option<&'static [Cell<u8>]>,
	/// if a bit index is greater then max_bits, then its invalid
	max_bits: usize,
}

// struct LookUpTable {
// }

struct SlaveInner {
	bitmap: [Bitmap; BUDDY_MAX_ORDER],
	/// reserved bytes of the bitmap.
	bitmap_reserved: usize,
	/// base addr of the bitmap, usually points to the bitmap.
	bitmap_base: PhysAddr,
	/// max supported order of the bitmap. any higher doesn't exist.
	max_order: Order,

	free: [Option<NonNull<PageFrame>>; BUDDY_MAX_ORDER],
	order_block_count: [u64; BUDDY_MAX_ORDER],
}

unsafe impl Send for SlaveInner {}

pub struct Slave {
	// free: [Option<NonNull<PageFrame>>; BUDDY_MAX_ORDER],
	// order_block_count: [u64; BUDDY_MAX_ORDER],

	// lut: LookUpTable,
	/// base of the whole region
	rbase: PhysAddr,
	/// base of the usable region
	mbase: PhysAddr,
	/// do total_pages << 12 (multiply by 4096) to get bytes.
	/// I don't believe this excludes reserved, so subtract lut.reserved
	total_pages: usize,

	inner: KMutex<SlaveInner>,
}

pub struct Master {
	// pub regions: [Option<NonNull<Slave>>; BUDDY_MAX_REGIONS],
	pub regions: [KOnce<&'static Slave>; BUDDY_MAX_REGIONS],
	add_lck: KMutex<()>,

	/// this isn't really needed, regions is terminated via None, but its convenient.
	pub region_count: AtomicUsize,
	// pub region_count: usize,
	/// This includes holes like NVS, BADRAM and reserved. This is mainly
	/// for gauging how large the physmap will be.
	pub ram_length: usize,
	/// usable ram is self explanatory, only ram regions marked available.
	pub usable_ram: usize,
}

pub static MASTER_BUDDY: KOnce<Master> = KOnce::new();

#[derive(Clone, Copy)]
pub struct ValidBit<'a> {
	byte: &'a Cell<u8>,
	mask: u8,
}


impl ValidBit<'_> {
	pub fn get(&self) -> bool {
		self.byte.get() & self.mask != 0
	}
	pub fn set(&self) {
		self.byte.set(self.byte.get() | self.mask)
	}
	pub fn clr(&self) {
		self.byte.set(self.byte.get() & !self.mask)
	}
}

impl SlaveInner {
	const fn new() -> Self {
		unsafe { zeroed() }
	}

	pub fn bit_check<'a>(
		&self,
		pfn: PageFrameNumber,
		order: Order,
	) -> Result<ValidBit<'a>, BuddyErr> {
		if order > self.max_order {
			return Err(InvalidBit);
		}

		let pidx = pfn.pair_index(order);
		let bitmap = &self.bitmap[order as usize];

		if pidx >= bitmap.max_bits {
			return Err(InvalidBit);
		};
		let Some(data) = bitmap.data else {
			return Err(InvalidBitmap);
		};

		Ok(ValidBit {
			byte: &data[pidx >> 3],
			mask: 1u8 << (pidx & (u8::BITS - 1) as usize),
		})
	}
}


impl Slave {
	const fn new() -> Self {
		Self {
			rbase: PhysAddr::new(0),
			mbase: PhysAddr::new(0),
			total_pages: 0,
			inner: KMutex::new(SlaveInner::new()),
		}
	}

	#[cold]
	/// # Safety
	/// this zeroes out all of the memory bru
	unsafe fn mark_invalid(&mut self, phys: PhysAddr) {
		unsafe {
			let region = self as *mut Slave;

			region.write(zeroed());
			(*region).total_pages = 0;
			(*region).rbase = phys;
		}
	}

	fn push(
		&self,
		inner: &mut KGuard<SlaveInner>,
		phy: PhysAddr,
		order: Order,
	) -> Result<(), BuddyErr> {
		let pfn = PageFrameNumber::get(phy, self);
		let bit = inner.bit_check(pfn, order)?;
		bit.set();

		let frame = unsafe { phy.to_phmap().as_mut_ptr::<PageFrame>().as_mut_unchecked() };

		frame.order = order;
		frame.next = inner.free[order as usize];

		inner.free[order as usize] = Some(NonNull::from_mut(frame));
		inner.order_block_count[order as usize] += 1;

		Ok(())
	}

	fn pop(
		&self,
		inner: &mut KGuard<SlaveInner>,
		phy: PhysAddr,
		order: Order,
	) -> Result<(), BuddyErr> {
		let pfn = PageFrameNumber::get(phy, self);
		let order_n = order as usize;
		let bit = inner.bit_check(pfn, order)?;
		let count = inner.order_block_count[order_n];

		if !phy.as_usize().is_multiple_of(PAGESIZE) {
			return Err(Misaligned);
		}

		let p = unsafe { NonNull::new_unchecked(phy.to_phmap().as_mut_ptr()) };

		let start = self.mbase.as_usize();
		let end = start + (self.total_pages << 12) - inner.bitmap_reserved;

		let ptr = p.as_ptr() as usize;
		if ptr < start || ptr >= end {
			return Err(InvalidParameters);
		}

		let mut i = 0;
		let mut cur = &mut inner.free[order_n];

		unsafe {
			if count == 0 {
				if cur.is_some() {
					cold_path();
					kernel_panic("buddy list corruption: list exists in free list!");
				}

				return Err(NoBlock);
			}

			while (*cur).is_some_and(|node| node != p) && i < count {
				// if cur ends up being None then the loop ends anyway.
				// The next check will catch it.
				cur = &mut (*(cur).unwrap_unchecked().as_ptr()).next;
				i += 1;
			}
		}

		match *cur {
		Some(n) if n == p && i < count => {
			if !bit.get() {
				cold_path();
				kernel_panic("buddy list corruption: mismatch between bitmap and free list!");
			}

			*cur = unsafe { (*n.as_ptr()).next };
			inner.order_block_count[order_n] -= 1;

			bit.clr();

			Ok(())
		},
		Some(_) => {
			cold_path();
			kernel_panic("buddy list corruption: end of list found after end of count!");
		},
		None if i < count => {
			cold_path();
			kernel_panic("buddy list corruption: end of list found before end of count!");
		},
		None => Err(NotInList),
		}
	}

	fn pop_head(&self, inner: &mut KGuard<SlaveInner>, order: Order) -> Result<*mut u8, BuddyErr> {
		let order_n = order as usize;
		let block = inner.free[order_n].ok_or(NoBlock)?;
		let phy = VirtAddr(block.as_ptr() as usize);

		let pfn = PageFrameNumber::get(phy.phmap_to_phys(), self);
		let bit = inner.bit_check(pfn, order)?;

		if !bit.get() {
			cold_path();
			kernel_panic("buddy list corruption: mismatch between bitmap and free list!");
		}

		inner.free[order_n] = unsafe { block.as_ref().next };
		inner.order_block_count[order_n] -= 1;

		bit.clr();
		Ok(block.as_ptr() as *mut u8)
	}
}

impl Master {
	#[cold]
	pub const fn new() -> Self {
		Self {
			regions: [const { KOnce::new() }; BUDDY_MAX_REGIONS],
			add_lck: KMutex::new(()),
			region_count: AtomicUsize::new(0),
			ram_length: 0,
			usable_ram: 0,
		}
	}

	fn add_region(&self, phy: PhysAddr, len: usize) -> Result<&Slave, BuddyErr> {
		for region in &self.regions {
			let r = region.get();
			if let Some(s) = r
				&& s.rbase == phy
			{
				return Err(AlreadyExists);
			}
		}

		let region_count = self.region_count.load(Ordering::Relaxed);
		if region_count + 1 > BUDDY_MAX_REGIONS {
			return Err(InvalidRegions);
		}

		init_region(phy, len)?;
		let vregion = phy.to_phmap().as_mut_ptr::<Slave>();
		let region = unsafe { vregion.as_ref_unchecked() };

		let _lck = self.add_lck.lock();

		for slot in self.regions.iter() {
			if slot.get().is_none() {
				slot.set(region).map_err(|_| BuddyErr::BadRegion)?;
				return Ok(region);
			}
		}

		Err(BuddyErr::MaxRegions)
	}
}

impl Default for Master {
	fn default() -> Self {
		Self::new()
	}
}


impl PageFrameNumber {
	fn get(phy: PhysAddr, s: &Slave) -> Self {
		PageFrameNumber((phy.as_usize() - s.mbase.as_usize()) >> 12)
	}

	fn pair_index(self, order: Order) -> usize {
		self.0 >> (order as usize + 1)
	}

	fn as_usize(self) -> usize {
		self.0
	}
}


#[inline]
fn order_index(n: usize) -> usize {
	n.trailing_zeros() as usize - 12_usize
}


fn find_slave_search(phys: PhysAddr) -> Option<&'static Slave> {
	let master = MASTER_BUDDY.get()?;
	let regions = master.region_count.load(Ordering::Relaxed);

	if regions == 0 || regions > BUDDY_MAX_REGIONS {
		core::hint::cold_path();
		return None;
	};

	let addr = phys.to_phmap().as_usize();

	for region in master.regions.iter().take(regions) {
		let r = region.get();
		if let Some(s) = r {
			let region = *(s);
			let region_start = region.rbase.as_usize();
			let region_end = region_start + (region.total_pages << 12);

			if addr >= region_start && addr < region_end {
				return Some(region);
			}
		} else {
			continue;
		}
	}

	None
}




fn find_slave(phys: PhysAddr) -> Option<&'static Slave> {
	let s = find_slave_search(phys)?;

	if phys.as_usize() >= s.mbase.as_usize() + (s.total_pages * PAGESIZE)
		|| phys.as_usize() < s.mbase.as_usize()
	{
		cold_path();
		None
	} else {
		Some(s)
	}
}



#[cold]
fn init_region(phy: PhysAddr, len: usize) -> Result<(), BuddyErr> {
	const BITMAP_ALIGN: usize = 0x40;
	if find_slave(phy).is_some() {
		return Err(AlreadyExists);
	};

	// let vregion: &'static mut Slave = unsafe { &mut *(phy.to_phmap().as_mut_ptr::<Slave>()) };
	// let vregion = phy.as_usize() as *const _ as KOnce<*const Slave>
	let vregion: *mut Slave = phy.as_usize() as *mut _;

	unsafe { (*vregion) = Slave::new() }

	let vregion = unsafe { vregion.as_mut_unchecked() };

	let rpages = page_align_down(len) >> 12;
	vregion.total_pages = rpages;
	vregion.rbase = phy;

	let mut vcur = (phy.as_usize() + size_of::<Slave>()).next_multiple_of(8);
	let mut g = vregion.inner.lock();

	for i in 0..BUDDY_MAX_ORDER {
		let bits: usize = rpages >> (i + 1);

		if bits == 0 {
			cold_path();
			break;
		}

		let bytes: usize = (bits + 7) >> 3_usize;

		let b: Bitmap = unsafe {
			Bitmap {
				data: Some(
					Cell::from_mut(from_raw_parts_mut(vcur as *mut u8, bytes)).as_slice_of_cells(),
				),
				max_bits: bits,
			}
		};

		g.bitmap[i] = b;
		g.max_order = Order::try_from(i).unwrap_or(Order::ORDER_INVALID);

		vcur += bytes.next_multiple_of(BITMAP_ALIGN);
	}

	let pcur = phy.as_usize() + (vcur - vregion as *const _ as usize);

	g.bitmap_base = PhysAddr(phy.as_usize() + size_of::<Slave>());
	vregion.mbase = PhysAddr(pcur.next_multiple_of(PAGESIZE));

	if vregion.mbase.as_usize() >= phy.as_usize() + len {
		cold_path();
		drop(g);
		unsafe { vregion.mark_invalid(phy) };
		return Err(BadRegion);
	}

	let pend = PhysAddr(phy.as_usize() + PAGESIZE);

	g.bitmap_reserved =
		if vregion.mbase > pend { (vregion.mbase.as_usize() - pend.as_usize()) >> 12 } else { 0 };

	vregion.total_pages = (phy.as_usize() + len - vregion.mbase.as_usize()) >> 12;

	let offset_len = (vcur - vregion as *const _ as usize) - size_of::<Slave>();
	let dst = (vregion as *const _ as usize + size_of::<Slave>()) as *mut u8;

	unsafe { dst.write_bytes(0, offset_len) };

	Ok(())
}


fn byte_order(sz: usize) -> u8 {
	let frames = sz.div_ceil(PAGESIZE);
	if frames <= 1 {
		return 0;
	};
	(((usize::BITS - 1) - (frames.saturating_sub(1)).leading_zeros()) + 1) as u8
}




pub fn buddy_free_range(vaddr: *mut u8, size_bytes: usize) -> Result<(), BuddyErr> {
	if size_bytes == 0 || ((vaddr as usize) & (PAGESIZE - 1)) != 0 {
		return Err(InvalidParameters);
	}

	let master = MASTER_BUDDY.get().ok_or(NotReady)?;

	let mut phy = PhysAddr(VirtAddr(vaddr as usize).phmap_to_phys().as_usize() & !(PAGESIZE - 1));

	// next multiple of would free bytes the caller didn't want to free.
	let mut size = size_bytes & !(PAGESIZE - 1);
	let mut s = find_slave(phy);

	if s.is_none() {
		// hopefully this doesnt leave regions in a corrupted state :P
		s = Some(master.add_region(phy, size_bytes)?);

		let rs = unsafe { s.unwrap_unchecked() };
		let offs = ((rs.mbase.as_usize() - rs.rbase.as_usize()) + (PAGESIZE - 1)) & !(PAGESIZE - 1);

		phy = PhysAddr(phy.as_usize() + offs);
		size -= offs;
	}

	let s = unsafe { s.unwrap_unchecked() };
	const max_order: usize = BUDDY_MAX_ORDER - 1;

	let mut g = s.inner.lock();
	while size > 0 {
		let len_order = ((usize::BITS - 1) - (size / PAGESIZE).leading_zeros()) as usize;
		let pfn = PageFrameNumber::get(phy, s);
		let ptr_order = pfn.as_usize().trailing_zeros() as usize;

		let order = ((len_order.min(ptr_order)).min(max_order)).min(g.max_order as usize);

		unsafe {
			// An error from this just means leaked pages. They're safe to leak, though not great.
			let _ = s.push(&mut g, phy, Order::try_from(order).unwrap_unchecked()).is_err();
		}

		let block_size = PAGESIZE << order;
		phy = PhysAddr(phy.as_usize() + block_size);
		size -= block_size;
	}

	Ok(())
}

/// Use buddy_free_range() for adding regions.
pub fn buddy_free(vaddr: *mut u8, order: Order) -> Result<(), BuddyErr> {
	if order == Order::ORDER_INVALID {
		return Err(InvalidParameters);
	};
	if !(vaddr as usize).is_multiple_of(PAGESIZE) {
		return Err(Misaligned);
	}

	// by proxy, vaddr's alignment has been guaranteed, so this is also aligned.
	let phy = VirtAddr(vaddr as usize).phmap_to_phys();
	let s = find_slave(phy).ok_or(NoBuddy)?;

	let mut pfn = PageFrameNumber::get(phy, s);
	let mut k = order as usize;

	if pfn.as_usize() & ((1_usize << order as usize) - 1) != 0 {
		return Err(Misaligned);
	}

	let mut g = s.inner.lock();

	while k < BUDDY_MAX_ORDER - 1 {
		let o = unsafe { Order::try_from(k).unwrap_unchecked() };
		let bpfn = pfn.as_usize() ^ (1_usize << k);

		if bpfn >= s.total_pages {
			break;
		}
		let Ok(bit) = g.bit_check(PageFrameNumber(bpfn), o) else {
			break;
		};
		if !bit.get() {
			break;
		};

		let buddy = PhysAddr(s.mbase.as_usize() + (bpfn << 12));

		s.pop(&mut g, buddy, o)
			.unwrap_or_else(|_| kernel_panic("buddy corruption: failure at pop in buddy_free()!"));

		pfn.0 &= !(1_usize << k);
		k += 1;
	}

	let o = unsafe { Order::try_from(k).unwrap_unchecked() };

	match s.push(&mut g, PhysAddr(s.mbase.as_usize() + (pfn.as_usize() << 12)), o) {
	Ok(_) => Ok(()),
	Err(e) if k <= order as usize => Err(e),
	Err(_) => kernel_panic("buddy corruption: push failure with k > order!"),
	}
}


pub fn buddy_alloc_range(size_bytes: usize) -> *mut u8 {
	if size_bytes == 0 {
		return null_mut();
	}

	let Some(master) = MASTER_BUDDY.get() else {
		return null_mut();
	};


	let size_rounded = size_bytes.next_multiple_of(PAGESIZE);
	let req = byte_order(size_rounded);

	if req >= BUDDY_MAX_ORDER as u8 {
		return null_mut();
	}

	// let regions = unsafe { from_raw_parts_mut(master.regions.as_mut_ptr(), master.region_count) };

	for region in &master.regions {
		let r = region.get();
		let Some(slave) = r else { continue };

		let mut g = slave.inner.lock();

		let mut order = req as usize;

		while order < BUDDY_MAX_ORDER
			&& order < g.max_order as usize
			&& g.order_block_count[order] == 0
		{
			order += 1;
		}

		if order >= BUDDY_MAX_ORDER || order < req as usize {
			continue;
		}

		let addr = unsafe { slave.pop_head(&mut g, Order::try_from(order).unwrap_unchecked()) };
		let Ok(addr) = addr else { continue };

		while order > req as usize {
			order -= 1;
			let buddy = PhysAddr(VirtAddr(addr as usize).phmap_to_phys().as_usize() ^ PAGESIZE << order);
			unsafe {
				let _ = slave.push(&mut g, buddy, Order::try_from(order).unwrap_unchecked());
			}
		}

		return addr;
	}

	null_mut()
}




pub fn buddy_alloc(order: Order) -> *mut u8 {
	if order == Order::ORDER_INVALID {
		return null_mut();
	}
	// TODO make alloc_range call alloc instead of the other way around
	buddy_alloc_range(order.val())
}
