use core::arch::asm;

#[inline(always)]
pub unsafe fn cld() {
    unsafe {
        asm!("cld", options(nomem, nostack));
    }
}
#[inline(always)]
pub unsafe fn std() {
    unsafe {
        asm!("std", options(nomem, nostack));
    }
}

