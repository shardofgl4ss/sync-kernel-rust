// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use crate::irq;
use core::panic::PanicInfo;
use crate::io::{hlt};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("rust panic called!");
    if let Some(loc) = info.location() {
        kprintln!("file: {f}, line: {l}, col: {c}",
            f = loc.file(),
            l = loc.line(),
            c = loc.column(),
        );
    }
    kprintln!("{}", info.message());
	unsafe { kernel_panic(info.message().as_str().unwrap_unchecked()) };
}

#[unsafe(no_mangle)]
pub fn kernel_panic(s: &str) -> ! {
    

    unsafe {
		irq::cli();
		loop {
			irq::cli();
			hlt();
		}
    }
}
