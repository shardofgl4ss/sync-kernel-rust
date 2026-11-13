// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::arch::asm;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::addr_of_mut;
use crate::idt::idt_flags::F_IDT_PRESENT;

pub enum idt_flags {
    F_IDT_GATE_TASK = 0x5,

    F_IDT_GATE_16BIT_IRQ = 0x6,
    F_IDT_GATE_16BIT_TRAP = 0x7,

    F_IDT_GATE_IRQ = 0xE,  // does clear interrupts automatically.
    F_IDT_GATE_TRAP = 0xF, // doesn't clear interrupts automatically

    F_IDT_RING_0 = 0 << 5, // kernelspace
    F_IDT_RING_1 = 1 << 5, // unused
    F_IDT_RING_2 = 2 << 5, // unused
    F_IDT_RING_3 = 3 << 5, // userspace

    F_IDT_PRESENT = 0x80,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct idt64_entry {
    offs_low: u16,
    segment_selector: u16,
    ist: u8,
    flags: u8,
    offs_mid: u16,
    offs_hi: u32,
    _reserved: u32,
}

#[repr(C, packed)]
struct interrupt_descriptor64 {
    limit: u16,
    base: *mut idt64_entry,
}

const _: () = {
    assert!(size_of::<idt64_entry>() == 0x10, "size check failed");
};
const _: () = {
    assert!(
        size_of::<interrupt_descriptor64>() == 10,
        "size check failed"
    );
};

const IDT_ENTRIES: usize = 256;

static mut IDT_ARRAY: [idt64_entry; IDT_ENTRIES] =
    [unsafe { zeroed::<idt64_entry>() }; IDT_ENTRIES];

static mut DESCRIPTOR: interrupt_descriptor64 = interrupt_descriptor64 {
    limit: (size_of::<[idt64_entry; IDT_ENTRIES]>() - 1) as u16,
    base: core::ptr::null_mut(),
};

#[inline(always)]
unsafe fn x64_idt_load() {
    unsafe {
        asm!("lidt [{dt}]",
            dt = in(reg) &raw mut DESCRIPTOR,
            options(nostack)
        );
    }
}

pub mod bits {
    #[inline(always)]
    pub fn extract_low(x: u64) -> u16 {
        (x & 0xFFFF) as u16
    }
    #[inline(always)]
    pub fn extract_mid(x: u64) -> u16 {
        ((x >> 16) & 0xFFFF) as u16
    }
    #[inline(always)]
    pub fn extract_high(x: u64) -> u32 {
        (x >> 32) as u32
    }
}

// These are all 64 bit IDTs.
pub unsafe fn setgate(interrupt: u8, base: *const c_void, seg_desc: u16, flags: u8) {
	unsafe {
		let idt: &mut idt64_entry = &mut IDT_ARRAY[interrupt as usize];

		idt.offs_low = bits::extract_low(base as u64);
		idt.offs_mid = bits::extract_mid(base as u64);
		idt.offs_hi = bits::extract_high(base as u64);

		idt.flags = flags;
		idt.segment_selector = seg_desc;
		idt.ist = 0;
		idt._reserved = 0;
	}
}

pub unsafe fn enablegate(interrupt: u8) {
	unsafe {
		let idt: &mut idt64_entry = &mut IDT_ARRAY[interrupt as usize];
		idt.flags |= F_IDT_PRESENT as u8;
	}
}


pub unsafe fn disablegate(interrupt: u8) {
	unsafe {
		let idt: &mut idt64_entry = &mut IDT_ARRAY[interrupt as usize];
		idt.flags &= !(F_IDT_PRESENT as u8);
	}
}

#[allow(static_mut_refs)]
pub fn init() {
	unsafe {
		let base : *mut idt64_entry = addr_of_mut!(IDT_ARRAY[0]);
		DESCRIPTOR.base = base;
		x64_idt_load();
	}
}
