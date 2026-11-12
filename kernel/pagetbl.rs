// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::ptr::NonNull;
use core::mem::size_of;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, IndexMut, Not};

use crate::include::types::{PhysAddr};
use crate::mm::buddy;

pub const PT_ENTRIES: usize = 512;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PTE(pub u64);


#[repr(align(0x1000))]
pub struct PageTable(pub [PTE; PT_ENTRIES]);

#[repr(align(0x1000))]
pub struct PageFrame {
    pub next: Option<NonNull<PageFrame>>,
    pub order: buddy::Order,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(pub u64);

impl PageFlags {
    pub const NONE: Self              = Self(0);
    pub const PRESENT: Self           = Self(1 << 0);
    pub const WRITABLE: Self          = Self(1 << 1);
    pub const USER: Self              = Self(1 << 2);
    pub const WRITE_THROUGH: Self     = Self(1 << 3);
    pub const PAGECACHE_DISABLE: Self = Self(1 << 4);
    pub const ACCESSED: Self          = Self(1 << 5);
    pub const DIRTY: Self             = Self(1 << 6);
    pub const PAGESIZE: Self          = Self(1 << 7);
    pub const GLOBAL: Self            = Self(1 << 8);
    pub const NX: Self                = Self(1 << 63);

    pub const READ_WRITE: Self        = Self(Self::PRESENT.0 | Self::WRITABLE.0);
    pub const HUGE: Self              = Self(Self::READ_WRITE.0 | Self::PAGESIZE.0);

    pub const fn contains(self, o: Self) -> bool { self.0 & o.0 == o.0 }
    pub const fn as_u64(self) -> u64 { self.0 }
}

impl PTE {
    pub const fn as_u64(self) -> u64 { self.0 }

    pub unsafe fn link(&mut self, table: *const PageTable, flags: PageFlags) {
        let entry = PTE(table as u64 | flags.as_u64());
        *self = entry;
    }

    // grabs the addr of the pagetable, sets it to zero and returns.
    /// # Safety
    /// This expects a valid table to exist there.
    pub unsafe fn unlink(&mut self)  -> Option<*mut PageTable> {
        if !self.is_present() || self.is_pagesize() {
            None
        } else {
            let table = pt_rebase(*self);
            *self = PTE(0u64);
            Some(table)
        }

    }

    pub const fn is_present(self) -> bool {
        self.is_set(PageFlags::PRESENT)
    }

    pub const fn is_pagesize(self) -> bool {
        self.is_set(PageFlags::PAGESIZE)
    }

    pub const fn is_set(self, flag: PageFlags) -> bool {
        (self.0 & flag.as_u64()) != 0
    }

    pub const fn is_unset(self, flag: PageFlags) -> bool {
        (self.0 & flag.as_u64()) == 0
    }

    pub const fn setflags(&mut self, flags: PageFlags) {
        self.0 |= flags.as_u64();
    }

    pub const fn unsetflags(&mut self, flags: PageFlags) {
        self.0 &= !flags.as_u64();
    }
}


impl BitOr for PageFlags {
    type Output = Self;
    fn bitor(self, o: Self) -> Self { Self(self.0 | o.0) }
}
impl BitAnd for PageFlags {
    type Output = Self;
    fn bitand(self, o: Self) -> Self { Self(self.0 & o.0) }
}
impl BitXor for PageFlags {
    type Output = Self;
    fn bitxor(self, o: Self) -> Self { Self(self.0 ^ o.0) }
}
impl Not for PageFlags {
    type Output = Self;
    fn not(self) -> Self { Self(!self.0) }
}

impl BitOrAssign for PageFlags {
    fn bitor_assign(&mut self, o: Self) { self.0 |= o.0 }
}
impl BitAndAssign for PageFlags {
    fn bitand_assign(&mut self, o: Self) { self.0 &= o.0 }
}
impl BitXorAssign for PageFlags {
    fn bitxor_assign(&mut self, o: Self) { self.0 ^= o.0 }
}


const _: () = assert!(size_of::<PTE>() == 0x8);
const _: () = assert!(size_of::<PageFrame>() == 0x1000);
const _: () = assert!(size_of::<PageTable>() == 0x1000);

pub const L1E_SIZE: usize = size_of::<PageTable>();
pub const L2E_SIZE: usize = L1E_SIZE * PT_ENTRIES;
pub const L3E_SIZE: usize = L2E_SIZE * PT_ENTRIES;
pub const L4E_SIZE: usize = L3E_SIZE * PT_ENTRIES;

pub const L2E_MASK: usize = L2E_SIZE - 1;
pub const L3E_MASK: usize = L3E_SIZE - 1;
pub const L4E_MASK: usize = L4E_SIZE - 1;

#[inline]
pub const fn bitrange_mask(hi: u32, lo: u32) -> u64 {
    (1u64 << (hi - lo + 1)) - 1u64
}

#[inline]
pub const fn page_phys_rebase(phys: u64, shift: u32, mask: u64) -> u64 {
    ((phys >> shift) & mask) << shift
}

const PT_ADDR_MASK: u64 = bitrange_mask(51, 12);

#[inline]
pub const fn pt_rebase(p: PTE) -> *mut PageTable {
    page_phys_rebase(p.as_u64(), 12, PT_ADDR_MASK) as *mut PageTable
}

#[inline(always)]
pub const fn l4_leaf(addr: usize) -> usize {
    (addr >> 39) & 0x1FF
}
#[inline(always)]
pub const fn l3_leaf(addr: usize) -> usize {
    (addr >> 30) & 0x1FF
}
#[inline(always)]
pub const fn l2_leaf(addr: usize) -> usize {
    (addr >> 21) & 0x1FF
}
#[inline(always)]
pub const fn l1_leaf(addr: usize) -> usize {
    (addr >> 12) & 0x1FF
}
#[inline(always)]
pub const fn page_offset(addr: usize) -> usize {
    addr & 0xFFF
}

impl PageTable {
    #[inline(always)]
    pub fn index_leaf(&mut self, idx: usize) -> &mut PTE {
        self.0.index_mut(idx)
    }
}

pub const PAGESIZE: usize = 0x1000usize;


#[inline(always)]
pub fn page_align_down(x: usize) -> usize {
    x & !(PAGESIZE - 1)
}
#[inline(always)]
pub fn page_align_up(x: usize) -> usize {
    (x + (PAGESIZE - 1)) & !(PAGESIZE - 1)
}


#[inline]
pub unsafe fn set_cr3(pml4: PhysAddr) {
    unsafe {
        core::arch::asm!(
            "movq   {}, %cr3",
            in(reg) pml4.as_usize(),
            options(att_syntax)
        );
    }
}

#[inline]
pub unsafe fn get_cr3() -> PhysAddr {
    let pml4: usize;
    unsafe {
        core::arch::asm!(
            "movq   %cr3, {}",
            out(reg) pml4,
            options(att_syntax)
        );
    }
    PhysAddr(pml4)
}

