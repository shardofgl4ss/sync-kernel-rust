// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#![allow(non_camel_case_types)]

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum ECX_Feat {
    ECX_SSE3        = 1 << 0,  // streaming SIMD extensions 3 (SSE3)
    ECX_MONITOR     = 1 << 3,  // monitor/mwait
    ECX_DS_CPL      = 1 << 4,  // CPL qualified debug store
    ECX_VMX         = 1 << 5,  // virtual machine extensions
    ECX_SMX         = 1 << 6,  // safer mode extensions
    ECX_EST         = 1 << 7,  // enhanced Intel SpeedStep(R) technology
    ECX_TM2         = 1 << 8,  // thermal monitor 2
    ECX_SSSE3       = 1 << 9,  // supplemental streaming SIMD extensions 3 (SSSSE3)
    ECX_CNXT_ID     = 1 << 10, // L1 context ID
    ECX_CMPXCHG16B  = 1 << 13, // cmpxchg16b available (obviously)
    ECX_xTPR_UPDATE = 1 << 14, // xTPR update control
    ECX_PDCM        = 1 << 15, // performance and debug capability
    ECX_DCA         = 1 << 18, // memory-mapped device prefetching
    ECX_SSE4_1      = 1 << 19, // SSE4.1
    ECX_SSE4_2      = 1 << 20, // SSE4.2
    ECX_x2APIC      = 1 << 21, // x2APIC available
    ECX_MOVBE       = 1 << 22, // movbe available
    ECX_POPCNT      = 1 << 23, // popcnt available (sounds rude)
    ECX_XSAVE       = 1 << 26, // xsave/xrstor/xsetbv/xgetbv supported
    ECX_OSXSAVE     = 1 << 27, // xsetbv/xgetbv has been enabled
}

pub enum EDX_Feat {
    EDX_x87   = 1 << 0,  // x86 FPU on chip
    EDX_VME   = 1 << 1,  // virtual-8086 mode enhancement
    EDX_DE    = 1 << 2,  // debugging extensions
    EDX_PSE   = 1 << 3,  // page size extensions
    EDX_TSC   = 1 << 4,  // timestamp counter
    EDX_MSR   = 1 << 5,  // rdmsr/wrmsr
    EDX_PAE   = 1 << 6,  // page address extensions
    EDX_MCE   = 1 << 7,  // machine check exception
    EDX_CX8   = 1 << 8,  // cmpxchg8b supported
    EDX_APIC  = 1 << 9,  // APIC on a chip
    EDX_SEP   = 1 << 11, // sysenter/sysexit
    EDX_MTRR  = 1 << 12, // memory type range registers
    EDX_PGE   = 1 << 13, // PTE global bit (PTE_GLOBAL)
    EDX_MCA   = 1 << 14, // machine check architecture
    EDX_CMOV  = 1 << 15, // conditional move/compare instructions
    EDX_PAT   = 1 << 16, // page attribute table
    EDX_PSE36 = 1 << 17, // page size extension
    EDX_PSN   = 1 << 18, // processor serial number
    EDX_CLFSH = 1 << 19, // cflush instruction
    EDX_DS    = 1 << 21, // debug store
    EDX_ACPI  = 1 << 22, // thermal monitor and clock control
    EDX_MMX   = 1 << 23, // MMX technology
    EDX_FXSR  = 1 << 24, // fxsave/fxrstor
    EDX_SSE   = 1 << 25, // SSE extensions
    EDX_SSE2  = 1 << 26, // SSE2 extensions, obviously
    EDX_SS    = 1 << 27, // self-snoop
    EDX_HTT   = 1 << 28, // multi-threading (hyper-threading, I think - the afterburner of Intel CPUs)
    EDX_TM    = 1 << 29, // thermal monitor
    EDX_PBE   = 1 << 31, // Pend. Brk. EN. (wtf?)
}

