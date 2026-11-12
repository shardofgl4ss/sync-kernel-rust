use core::arch::asm;

#[inline(always)]
pub unsafe fn cli() {
	unsafe {
		asm!("cli", options(nomem, nostack));
	}
}

#[inline(always)]
pub unsafe fn sti() {
	unsafe {
		asm!("sti", options(nomem, nostack));
	}
}
