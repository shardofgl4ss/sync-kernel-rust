// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

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

