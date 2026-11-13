// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

const KERNEL_OFFSET: usize = 0xFFFF_FFFF_8000_0000;
const KERNEL_PHMMAP: usize = 0xFFFF_8000_0000_0000;


impl PhysAddr {
    pub const fn new(a: usize) -> Self { Self(a) }
    pub const fn as_usize(self) -> usize { self.0 }

    pub const fn to_phmap(self) -> VirtAddr { VirtAddr(self.as_usize() + KERNEL_PHMMAP) }
    pub const fn to_kcode(self) -> VirtAddr { VirtAddr(self.as_usize() + KERNEL_OFFSET) }
    
    pub const fn is_invalid(self) -> bool { self.as_usize() == usize::MAX }
    pub const fn is_valid(self) -> bool { self.as_usize() != usize::MAX }
}

impl VirtAddr {
    pub const fn new(a: usize) -> Self { Self(a) }
    pub const fn as_usize(self) -> usize { self.0 }

    pub const fn as_ptr<T>(self) -> *const T { self.0 as *const T }
    pub const fn as_mut_ptr<T>(self) -> *mut T { self.0 as *mut T }

    pub const fn phmap_to_phys(self) -> PhysAddr { PhysAddr(self.as_usize() - KERNEL_PHMMAP) }
    pub const fn kcode_to_phys(self) -> PhysAddr { PhysAddr(self.as_usize() - KERNEL_OFFSET) }

    pub const fn is_invalid(self) -> bool { self.as_usize() == usize::MAX || self.as_usize() == 0_usize }
    pub const fn is_valid(self) -> bool { self.as_usize() != usize::MAX || self.as_usize() != 0_usize }
}

