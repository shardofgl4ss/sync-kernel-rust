// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::mem::zeroed;

use crate::include::types::{PhysAddr, VirtAddr};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KernelBootType {
    BIOS = 0,
    EFI32 = 1,
    EFI64 = 2,
}

#[derive(Clone, Copy)]
pub struct BootFramebuffer {
    pub phys: PhysAddr,
    pub virt: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
}

#[derive(Clone, Copy)]
pub struct BootMapEntry {
    pub base: PhysAddr,
    pub len: usize,
}

pub struct BootMemMap {
    pub entries: [BootMapEntry; 64],
    pub entries_recl: [BootMapEntry; 8],

    pub ram_max: usize,
    pub ram_usable: usize,

    pub count: u8,
    pub count_recl: u8,
}


pub struct KernelBootInfo {
    pub cmdline: [u8; 256],

    pub fb: BootFramebuffer,
    pub maps: BootMemMap,

    pub load_addr: PhysAddr,
    pub acpi_rsdp: PhysAddr,
    pub bootenv: KernelBootType,
}

pub static mut KERNEL_BOOT_INFO: KernelBootInfo = unsafe {zeroed()};

