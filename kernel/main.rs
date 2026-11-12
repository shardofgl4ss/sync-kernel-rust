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

mod init;
pub mod panic;
pub mod bootparams;
pub mod drivers;
pub mod pagetbl;
pub mod arch;
pub mod gdt;
pub mod isr;
pub mod irq;
pub mod idt;
pub mod mm;

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
