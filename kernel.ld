/*
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (C) 2026 Sync Shard
 */

ENTRY(_start)

KERNEL_OFFSET = 0xFFFFFFFF80000000; /* kernel code offset. */
KERNEL_VMMMAP = 0xFFFFC00000000000; /* io remapping base, 64TiB limit */
KERNEL_PHMMAP = 0xFFFF800000000000; /* direct physical mapping of all ram, 64TiB limit */

PHDRS
{
        preinit PT_LOAD FLAGS(5);
        pre_dat PT_LOAD FLAGS(6);
        text    PT_LOAD FLAGS(5);
        rodata  PT_LOAD FLAGS(4);
        data    PT_LOAD FLAGS(6);
}


SECTIONS
{
        . = 0x100000;

        _kphys_start = .;

        /* These sections are in an identity map, no AT() */
        __prebegin = .;

        .preinit : ALIGN(0x1000)
        {
                KEEP(*(.multiboot))
                KEEP(*(.preinit.text))
                KEEP(*(.preinit.text*))
        } :preinit

        .preinit.rodata : ALIGN(0x1000)
        {
                *(.preinit.rodata)
                *(.preinit.rodata*)
        } :preinit

        .preinit.data : ALIGN(0x1000)
        {
                *(.preinit.data)
                *(.preinit.data*)
        } :pre_dat

        .preinit.bss (NOLOAD) : ALIGN(0x1000)
        {
                *(.preinit.bss)
                *(.preinit.bss*)

        } :pre_dat

        .preinit.pt (NOLOAD) : ALIGN(0x1000)
        {
                pagetable_L4 = .;
                . += 0x1000;
                pagetable_L3 = .;
                . += 0x2000; /* same as below, but only one needed for kcode map */
                pagetable_L2 = .;
                . += 0x3000; /* one for the ident map, two for the temp kcode map */
        } :pre_dat

        __preend = .;

        . = ALIGN(0x1000);
        _KERN_LMA = .;
        _OFFS = 0x0;

        . += KERNEL_OFFSET;
        _KERN_VMA = .;
        _begin = .;

        .text ALIGN(0x1000) : AT(_KERN_LMA)
        {
                *(.text)
                *(.text.*)
        } :text

        _OFFS = SIZEOF(.text);
        _OFFS = ALIGN(_OFFS, 0x1000);

        .rodata ALIGN(0x1000) : AT(_KERN_LMA + _OFFS)
        {
                *(.rodata)
                *(.rodata.*)
        } :rodata

        _after_ro = .;

        _OFFS += SIZEOF(.rodata);
        _OFFS = ALIGN(_OFFS, 0x1000);

        .data ALIGN(0x1000) : AT(_KERN_LMA + _OFFS)
        {
                __data_start = .;

                *(.data)
                *(.data.*)

                . = ALIGN(0x10);

                *(.got)
                *(.got.*)
                *(.got.plt)

                __data_end = .;
        } :data

        _OFFS += SIZEOF(.data);
        _OFFS = ALIGN(_OFFS, 0x1000);

        .bss ALIGN(0x1000) : AT(_KERN_LMA + _OFFS)
        {
                __bss_start = .;

                *(COMMON)
                *(.bss)
                *(.bss.*)

                __bss_end = .;
        } :data

        _OFFS += SIZEOF(.bss);
        _OFFS = ALIGN(_OFFS, 0x1000);

        _end = .;
        . = ALIGN(0x1000);
        . += 0x1000;
        kstack_guard = .;

        . += 16K;
        kstack_top = .;

        . = ALIGN(0x1000);
        . += 0x1000;
        _kphys_end = . - KERNEL_OFFSET;

        /DISCARD/ :
        {
                *(.eh_frame)
                *(.eh_frame_hdr)
        }
}
