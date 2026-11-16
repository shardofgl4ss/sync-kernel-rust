/*
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (C) 2026 Sync Shard
 */

.set MAGIC,     0xE85250D6
.set MB_SIZE,   mb_end - mb_start


.extern _begin
.type _begin, @object
.extern start_kernel
.type start_kernel, @function


.align 8
.section .multiboot
mb_start:
        .long MAGIC
        .long 0
        .long MB_SIZE
        .long -(MAGIC + 0 + MB_SIZE)

        .short 0
        .short 0
        .long 8
mb_end:
        

.section .preinit.bss, "aw", @nobits
        .align 16
tstack_bottom:
        .skip 0x4000
tstack_top:


.section .preinit.text, "ax", @progbits
        .globl _start
        .type _start, @function
.code32
_start:
        mov     %ebx, (multiboot2_struct)

        lgdt    (GDT_descriptor)

        .cfi_startproc
        movw    $DATA_SEGMENT, %ax
        movw    %ax, %ds
        movw    %ax, %es
        movw    %ax, %fs
        movw    %ax, %gs
        movw    %ax, %ss

        call    check_cpuid
        call    check_longmode

        .cfi_endproc
enter_long:
        .cfi_startproc
        call    setup_vmpages
        call    enable_paging

        lgdt    (GDT64_descriptor)

	# movb    $'A', %al
	# outb    %al, $0xe9

        jmp     $CODE_SEGMENT64, $long_mode_start

        .cfi_endproc
.code64
long_mode_start:
        .cfi_startproc

        movw    $0x0, %ax
        movw    %ax, %ds
        movw    %ax, %es
        movw    %ax, %ss
        movw    %ax, %fs
        movw    %ax, %gs

        movq    $tstack_top, %rsp
        andq    $-16, %rsp
        xorq    %rbp, %rbp

        call    setup_higherhalf

        movq    (multiboot2_struct), %rdi
        jmp     start_kernel

        cli
        hlt
        .cfi_endproc
.size _start, . - _start


.set PT_FLAGS,          ((1 << 0) | (1 << 1))            # 0b00000011 present, writable
.set PT_FLAGS_HP,       ((1 << 0) | (1 << 1) | (1 << 7)) # 0b10000011 present, writable, hugepage
.set PT_MAPSZ,          (1 << 21)                        # 2MiB

.extern KERNEL_OFFSET
.type KERNEL_OFFSET, @object

.extern pagetable_L4
.extern pagetable_L3
.extern pagetable_L2


setup_higherhalf:
        movq    $pagetable_L3+0x1000, %rdx
        movq    $pagetable_L2+0x1000, %rsi
        movq    $pagetable_L4, %rdi

        movq    %rdx, %rax
        orq     $PT_FLAGS, %rax
        movq    %rax, 511*8(%rdi)

        xorq    %rcx, %rcx
.Lhh_loop:
        movq    %rcx, %rax
        shlq    $21, %rax       # 2MiB shift
        orq     $PT_FLAGS_HP, %rax
        movq    %rax, (%rsi, %rcx, 8)
        inc     %rcx
        cmpq    $(512*2), %rcx
        jne     .Lhh_loop

        movq    %rsi, %rax
        orq     $PT_FLAGS, %rax
        movq    %rax, 510*8(%rdx)
        addq    $0x1000, %rax
        movq    %rax, 511*8(%rdx)

        ret


.code32
setup_vmpages:
        movl    $pagetable_L3, %eax
        orl     $PT_FLAGS, %eax
        movl    %eax, (pagetable_L4)

        movl    $pagetable_L2, %eax
        orl     $PT_FLAGS, %eax
        movl    %eax, (pagetable_L3)

        xorl    %ecx, %ecx
.Lsetup_vmpages_loop:   # mapping a whole page table

        movl    $PT_MAPSZ, %eax
        mul     %ecx
        orl     $PT_FLAGS_HP, %eax
        movl    %eax, pagetable_L2(, %ecx, 8)

        incl    %ecx
        cmpl    $0x200, %ecx
        jne     .Lsetup_vmpages_loop

        ret


enable_paging:
        movl    $pagetable_L4, %eax
        movl    %eax, %cr3

        # physical address extension (PAE)
        movl    %cr4, %eax
        orl     $(1 << 5), %eax # PAE bit
        orl     $(1 << 9), %eax # FXSAVE/FXRSTOR
        movl    %eax, %cr4 # PAE is now enabled.

        # long mode
        movl    $0xC0000080, %ecx # magic value for long mode
        rdmsr   # loads %efer register into %eax
        orl     $(1 << 8), %eax # long mode bit
        wrmsr   # long mode is now enabled.

        # finally enabling paging
        movl    %cr0, %eax
        orl     $(1 << 31), %eax # paging bit
        andl    $~(1 << 2), %eax # emulation bit (off)
        movl    %eax, %cr0

        ret




# Prints a stub boot error to VGA, then halts.
# parameters:
#       - %edi: string address
#       - %esi: len
# returns:
#       - does not return
.set VGA_LEN, 80
stub_perror:
        cli
        subl    $0x10, %esp
        push    %edi
        push    %esi

        movl    $0xb8000, %edi
        movl    $VGA_LEN, %ecx
.Lclear_line:
        movb    $0, (%edi)
        movb    $0x00, 1(%edi)
        addl    $2, %edi

        dec     %ecx
        jnz     .Lclear_line



        movl    $0xb8000, %edi
        movl    $stub_err, %esi
        movl    $stub_err_len, %ecx
        xor     %ah, %ah
.Lloop:
        movb    (%esi), %al
        movb    %al, (%edi)
        movb    $0x07, 1(%edi)
        addl    $2, %edi
        incl    %esi

        dec     %ecx
        jnz     .Lloop

        pop     %ecx
        pop     %esi

	# movb    $'A', %al
	# outb    %al, $0xe9

        cmp     $1, %ah
        mov     $1, %ah
        jne     .Lloop


1:
        hlt
        jmp     1b
.size stub_perror, . - stub_perror

check_cpuid:
        pushfl
        pop     %eax
        mov     %eax, %ecx
        xorl    $(1 << 21), %eax
        push    %eax

        popfl
        pushfl

        pop     %eax
        push    %ecx
        popfl

        cmp     %eax, %ecx
        je      .Lno_cpuid
        ret

.Lno_cpuid:
        movl    $cpuid_err, %edi
        mov     $cpuid_err_len, %esi
        jmp     stub_perror
.size check_cpuid, . - check_cpuid

check_longmode:
        movl    $0x80000000, %eax
        cpuid

        cmpl    $0x80000001, %eax
        jb      .Lno_longmode

        movl    $0x80000001, %eax
        cpuid
        testl   $(1 << 29), %edx
        jz      .Lno_longmode
        ret

.Lno_longmode:
        movl    $longmode_err, %edi
        movl    $longmode_err_len, %esi
        jmp     stub_perror
.size check_longmode, . - check_longmode

.section .preinit.bss, "aw", @nobits
        .globl multiboot2_struct
        .type multiboot2_struct, @object
        multiboot2_struct: .space 0x8
        .size multiboot2_struct, . - multiboot2_struct

.section .preinit.rodata, "a", @progbits

        stub_err:       .ascii "BOOT ERR: "
        .set stub_err_len, . - stub_err
        cpuid_err:      .ascii "cpuid not supported"
        .set cpuid_err_len, . - cpuid_err
        longmode_err:   .ascii "64 bit mode not supported"
        .set longmode_err_len, . - longmode_err



# GDT common access bits, both CS and DS
.set GDT_PRESENT,               (1 << 7)                # 0b10000000 P: segment is loaded in memory
.set GDT_DPL_RING0,             0                       # 0b00000000 DPL 0: kernelspace
.set GDT_DPL_RING3,             ((1 << 5) | (1 << 6))   # 0b01100000 DPL 3: userspace
.set GDT_DESC_S,                (1 << 4)                # 0b00010000 S: 1 is code/data, 0 is system (tss, ldt gates, etc)
.set GDT_ACCESSED,              (1 << 0)                # 0b00000001 used by cpu

# %ds bytes
.set GDT_DS_DIRECTION,          (1 << 2)        # 0b00000100 expand-down data segment
.set GDT_DS_WRITABLE,           (1 << 1)        # 0b00000010 1 = RW, 0 = RO

# %cs access bytes
.set GDT_CS_EXECUTABLE,         (1 << 3)        # 0b00001000 self explanatory
.set GDT_CS_CONFORMING,         (1 << 2)        # 0b00000100 allows execution without changing cpl
.set GDT_CS_READABLE,           (1 << 1)        # 0b00000010 self explanatory


# flags
.set GDT_FLAG_GRANULARITY,      (1 << 3)        # 0b1000 G: limit becomes 4KB pages instead of bytes, 4gb limit
.set GDT_FLAG_32BIT,            (1 << 2)        # 0b0100 D/B: 32 bit code/data
.set GDT_FLAG_LONG_MODE,        (1 << 1)        # 0b0010 L: 64 bit mode
.set GDT_FLAG_AVL,              (1 << 0)        # 0b0001 flag for os


# data segment is specified by leaving GDT_CS_EXECUTABLE not set.

.set GDT_KERNEL_CS_PPT,         (GDT_PRESENT | GDT_DPL_RING0 | GDT_DESC_S | GDT_CS_EXECUTABLE | GDT_CS_READABLE)
.set GDT_KERNEL_DS_PPT,         (GDT_PRESENT | GDT_DPL_RING0 | GDT_DESC_S | GDT_DS_WRITABLE)
.set GDT_KERNEL_FLAGS,          (GDT_FLAG_GRANULARITY | GDT_FLAG_32BIT)

# .set GDT64_KERNEL_CS_PPT,       (((GDT_PRESENT | GDT_DPL_RING0 | GDT_DESC_S | GDT_CS_EXECUTABLE | GDT_CS_READABLE) << 40) | (GDT_FLAG_LONG_MODE << 52))
# .set GDT64_KERNEL_DS_PPT,       (GDT_PRESENT | GDT_DPL_RING0 | GDT_DESC_S | GDT_CS_EXECUTABLE | GDT_CS_READABLE)

.set GDT_LIMIT_LO,              0xffff
.set GDT_LIMIT_HI,              0b1111




##### 32 bit GDT #####
.align 4
GDT_start:
        gdt_null_desc:  .quad 0

        gdt_code_desc:  .word GDT_LIMIT_LO      # first 16 bits of the limit
                        .word 0                 # base has to be 24 bits, so
                        .byte 0                 # word (16) + byte (8) = 24
                        .byte GDT_KERNEL_CS_PPT # pres,priv,type
                        .byte ((GDT_KERNEL_FLAGS << 4) | GDT_LIMIT_HI) # other + limit
                        .byte 0

        gdt_data_desc:  .word GDT_LIMIT_LO      # same as above, just with DS PPT instead.
                        .word 0
                        .byte 0
                        .byte GDT_KERNEL_DS_PPT
                        .byte ((GDT_KERNEL_FLAGS << 4) | GDT_LIMIT_HI)
                        .byte 0
GDT_end:

GDT_descriptor:
        .word GDT_end - GDT_start - 1
        .long GDT_start

.set CODE_SEGMENT,      gdt_code_desc - GDT_start
.set DATA_SEGMENT,      gdt_data_desc - GDT_start





.code64
##### 64 bit GDT #####
.align 4
GDT64_start:
        gdt64_null_desc:        .quad 0

        gdt64_code_desc:        .word 0
                                .word 0
                                .byte 0
                                .byte GDT_KERNEL_CS_PPT
                                .byte (GDT_FLAG_LONG_MODE << 4)
                                .byte 0
GDT64_end:

GDT64_descriptor:
        .word GDT64_end - GDT64_start - 1
        .quad GDT64_start

.set CODE_SEGMENT64,    gdt64_code_desc - GDT64_start
