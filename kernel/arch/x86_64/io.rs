// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::arch::asm;

#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
	unsafe {
		asm!("outb %al, %dx",
		in("dx") port,
		in("al") val,
		options(att_syntax, nomem, nostack),
		);
	}
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
	let ret: u8;
	unsafe {
		asm!("inb %dx, %al",
		in("dx") port,
		lateout("al") ret,
		options(att_syntax, nomem, nostack)
		);
	}
	ret
}

#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
	unsafe {
		asm!("outw %ax, %dx",
		in("dx") port,
		in("ax") val,
		options(att_syntax, nomem, nostack)
		);
	}
}

#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
	let ret: u16;
	unsafe {
		asm!("inw %dx, %ax",
		in("dx") port,
		lateout("ax") ret,
		options(att_syntax, nomem, nostack)
		);
	}
	ret
}

#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
	unsafe {
		asm!("outl %eax, %dx",
		in("dx") port,
		in("eax") val,
		options(att_syntax, nomem, nostack)
		);
	}
}

#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
	let ret: u32;
	unsafe {
		asm!("inl %dx, %eax",
		in("dx") port,
		lateout("eax") ret,
		options(att_syntax, nomem, nostack)
		);
	}
	ret
}

#[inline(always)]
pub unsafe fn insb(port: u16, out: *mut u8, wc: usize) {
    unsafe {
        asm!("cld",
        "rep insb",
        in("dx") port,
        inout("rcx") wc => _,
        inout("rdi") out => _,
        options(att_syntax, nostack)
        );
    }
}

#[inline(always)]
pub unsafe fn insw(port: u16, out: *mut u16, wc: usize) {
    unsafe {
        asm!("cld",
        "rep insw",
        in("dx") port,
        inout("rcx") wc => _,
        inout("rdi") out => _,
        options(att_syntax, nostack)
        );
    }
}

#[inline(always)]
pub unsafe fn insl(port: u16, out: *mut u32, wc: usize) {
    unsafe {
        asm!("cld",
        "rep insl",
        in("dx") port,
        inout("rcx") wc => _,
        inout("rdi") out => _,
        options(att_syntax, nostack)
        );
    }
}


#[inline(always)]
pub unsafe fn hlt() {
	unsafe {
		asm!("hlt", options(att_syntax, nomem, nostack));
	}
}
