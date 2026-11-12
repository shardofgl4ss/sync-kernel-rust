// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::{
    cell::Cell,
    fmt,
};
use crate::{
    stdio::vsprintf,
    arch::x86_64::io::{outb},
};


struct KString {
    len: u16,
    chars: [u8],
}

const KWRITE_BUFFER_CAPACITY: usize = 4096;

struct KWriteRing<const N: usize> {
    buffer: [Cell<u8>; N],

    head: Cell<usize>,
    tail: Cell<usize>,
    dropped: Cell<usize>,
}


fn isa_debug_print(byte_str: &[u8]) {
    const ISA_DEBUGCON: u16 = 0xE9;

    unsafe {
        for ch in byte_str {
            outb(ISA_DEBUGCON, *ch);
        }
    }
}


pub fn vprintk(args: fmt::Arguments<'_>) {
    let mut buf = [0u8; 1024];

    if let Ok(s) = vsprintf(&mut buf, args) {
        isa_debug_print(s.as_bytes());
    }
}


#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => { $crate::printk::vprintk(format_args!("{}\n", format_args!($($arg)*))) };
}

