// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::enum_clike_unportable_variant)]
#![allow(nonstandard_style)]
#![allow(dead_code)]

pub mod include;

use crate::arch::x86_64::io;

#[macro_use]
pub mod stdio;
#[macro_use]
pub mod printk;

pub mod arch;
pub mod bootparams;
pub mod drivers;
// hehe sync
pub mod gdt;
pub mod idt;
mod init;
pub mod irq;
pub mod isr;
pub mod mm;
pub mod pagetbl;
pub mod panic;
pub mod sync;

unsafe fn kidle() -> ! {
	unsafe {
		loop {
			io::hlt();
		}
	}
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
	mm::kmalloc::init();
	gdt::init();
	isr::init();
	idt::init();


	unsafe { kidle() };
}
