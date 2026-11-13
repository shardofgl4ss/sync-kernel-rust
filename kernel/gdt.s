# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 Sync Shard

.global kern_gdt_init
.type kern_gdt_init, @function

.extern CODE_SEGMENT64
.type CODE_SEGMENT64, @object


kern_gdt_init:
        .cfi_startproc
        push    %rbp
        mov     %rsp, %rbp

        lgdt    (%rdi)

        pushq   CODE_SEGMENT64(%rip)
        leaq    after(%rip), %rax
        pushq   %rax

        lretq
after:
        leave
        ret
        .cfi_endproc
