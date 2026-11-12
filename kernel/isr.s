.code64

.macro ISR_NOERR irq
        .global x64_ISR\irq
        .type x64_ISR\irq, @function
        x64_ISR\irq:
                pushq   $0
                pushq   $(\irq)
                jmp     isr_common
        .size x64_ISR\irq, . - x64_ISR\irq
.endm
.macro ISR_ERROR irq
        .global x64_ISR\irq
        .type x64_ISR\irq, @function
        x64_ISR\irq:
                pushq   $\irq # cpu pushes error on the stack, so no need
                jmp     isr_common
        .size x64_ISR\irq, . - x64_ISR\irq
.endm

        
.section .text.isr

ISR_NOERR 0     # divide by zero
ISR_NOERR 1     # debug
ISR_NOERR 2     # non-maskable interrupt
ISR_NOERR 3     # breakpoint
ISR_NOERR 4     # overflow
ISR_NOERR 5     # bound range exceeded
ISR_NOERR 6     # invalid opcode
ISR_NOERR 7     # device not available
ISR_ERROR 8     # ERR: double fault
ISR_NOERR 9     # coprocessor segment overrun (reserved)
ISR_ERROR 10    # ERR: invalid TSS
ISR_ERROR 11    # ERR: no segment
ISR_ERROR 12    # ERR: stack fault
ISR_ERROR 13    # ERR: general protection fault
ISR_ERROR 14    # ERR: page fault
ISR_NOERR 15    # reserved by intel
ISR_NOERR 16    # x87 FPU error
ISR_ERROR 17    # ERR: alignment chk
ISR_NOERR 18    # machine chk
ISR_NOERR 19    # SIMD/AVX floating point exception
ISR_NOERR 20    # virtualization error
ISR_ERROR 21    # control protection exception
ISR_NOERR 22    # 22-27 reserved
ISR_NOERR 23    # ...
ISR_NOERR 24    # ...
ISR_NOERR 25    # ...
ISR_NOERR 26    # ...
ISR_NOERR 27    # 22-27 reserved
ISR_NOERR 28    # hypervisor injection
ISR_ERROR 29    # ERR: VMM comm
ISR_ERROR 30    # security exception
ISR_NOERR 31    # the rest here on out are reserved.
ISR_NOERR 32
ISR_NOERR 33
ISR_NOERR 34
ISR_NOERR 35
ISR_NOERR 36
ISR_NOERR 37
ISR_NOERR 38
ISR_NOERR 39
ISR_NOERR 40
ISR_NOERR 41
ISR_NOERR 42
ISR_NOERR 43
ISR_NOERR 44
ISR_NOERR 45
ISR_NOERR 46
ISR_NOERR 47
ISR_NOERR 48
ISR_NOERR 49
ISR_NOERR 50
ISR_NOERR 51
ISR_NOERR 52
ISR_NOERR 53
ISR_NOERR 54
ISR_NOERR 55
ISR_NOERR 56
ISR_NOERR 57
ISR_NOERR 58
ISR_NOERR 59
ISR_NOERR 60
ISR_NOERR 61
ISR_NOERR 62
ISR_NOERR 63
ISR_NOERR 64
ISR_NOERR 65
ISR_NOERR 66
ISR_NOERR 67
ISR_NOERR 68
ISR_NOERR 69
ISR_NOERR 70
ISR_NOERR 71
ISR_NOERR 72
ISR_NOERR 73
ISR_NOERR 74
ISR_NOERR 75
ISR_NOERR 76
ISR_NOERR 77
ISR_NOERR 78
ISR_NOERR 79
ISR_NOERR 80
ISR_NOERR 81
ISR_NOERR 82
ISR_NOERR 83
ISR_NOERR 84
ISR_NOERR 85
ISR_NOERR 86
ISR_NOERR 87
ISR_NOERR 88
ISR_NOERR 89
ISR_NOERR 90
ISR_NOERR 91
ISR_NOERR 92
ISR_NOERR 93
ISR_NOERR 94
ISR_NOERR 95
ISR_NOERR 96
ISR_NOERR 97
ISR_NOERR 98
ISR_NOERR 99
ISR_NOERR 100
ISR_NOERR 101
ISR_NOERR 102
ISR_NOERR 103
ISR_NOERR 104
ISR_NOERR 105
ISR_NOERR 106
ISR_NOERR 107
ISR_NOERR 108
ISR_NOERR 109
ISR_NOERR 110
ISR_NOERR 111
ISR_NOERR 112
ISR_NOERR 113
ISR_NOERR 114
ISR_NOERR 115
ISR_NOERR 116
ISR_NOERR 117
ISR_NOERR 118
ISR_NOERR 119
ISR_NOERR 120
ISR_NOERR 121
ISR_NOERR 122
ISR_NOERR 123
ISR_NOERR 124
ISR_NOERR 125
ISR_NOERR 126
ISR_NOERR 127
ISR_NOERR 128
ISR_NOERR 129
ISR_NOERR 130
ISR_NOERR 131
ISR_NOERR 132
ISR_NOERR 133
ISR_NOERR 134
ISR_NOERR 135
ISR_NOERR 136
ISR_NOERR 137
ISR_NOERR 138
ISR_NOERR 139
ISR_NOERR 140
ISR_NOERR 141
ISR_NOERR 142
ISR_NOERR 143
ISR_NOERR 144
ISR_NOERR 145
ISR_NOERR 146
ISR_NOERR 147
ISR_NOERR 148
ISR_NOERR 149
ISR_NOERR 150
ISR_NOERR 151
ISR_NOERR 152
ISR_NOERR 153
ISR_NOERR 154
ISR_NOERR 155
ISR_NOERR 156
ISR_NOERR 157
ISR_NOERR 158
ISR_NOERR 159
ISR_NOERR 160
ISR_NOERR 161
ISR_NOERR 162
ISR_NOERR 163
ISR_NOERR 164
ISR_NOERR 165
ISR_NOERR 166
ISR_NOERR 167
ISR_NOERR 168
ISR_NOERR 169
ISR_NOERR 170
ISR_NOERR 171
ISR_NOERR 172
ISR_NOERR 173
ISR_NOERR 174
ISR_NOERR 175
ISR_NOERR 176
ISR_NOERR 177
ISR_NOERR 178
ISR_NOERR 179
ISR_NOERR 180
ISR_NOERR 181
ISR_NOERR 182
ISR_NOERR 183
ISR_NOERR 184
ISR_NOERR 185
ISR_NOERR 186
ISR_NOERR 187
ISR_NOERR 188
ISR_NOERR 189
ISR_NOERR 190
ISR_NOERR 191
ISR_NOERR 192
ISR_NOERR 193
ISR_NOERR 194
ISR_NOERR 195
ISR_NOERR 196
ISR_NOERR 197
ISR_NOERR 198
ISR_NOERR 199
ISR_NOERR 200
ISR_NOERR 201
ISR_NOERR 202
ISR_NOERR 203
ISR_NOERR 204
ISR_NOERR 205
ISR_NOERR 206
ISR_NOERR 207
ISR_NOERR 208
ISR_NOERR 209
ISR_NOERR 210
ISR_NOERR 211
ISR_NOERR 212
ISR_NOERR 213
ISR_NOERR 214
ISR_NOERR 215
ISR_NOERR 216
ISR_NOERR 217
ISR_NOERR 218
ISR_NOERR 219
ISR_NOERR 220
ISR_NOERR 221
ISR_NOERR 222
ISR_NOERR 223
ISR_NOERR 224
ISR_NOERR 225
ISR_NOERR 226
ISR_NOERR 227
ISR_NOERR 228
ISR_NOERR 229
ISR_NOERR 230
ISR_NOERR 231
ISR_NOERR 232
ISR_NOERR 233
ISR_NOERR 234
ISR_NOERR 235
ISR_NOERR 236
ISR_NOERR 237
ISR_NOERR 238
ISR_NOERR 239
ISR_NOERR 240
ISR_NOERR 241
ISR_NOERR 242
ISR_NOERR 243
ISR_NOERR 244
ISR_NOERR 245
ISR_NOERR 246
ISR_NOERR 247
ISR_NOERR 248
ISR_NOERR 249
ISR_NOERR 250
ISR_NOERR 251
ISR_NOERR 252
ISR_NOERR 253
ISR_NOERR 254
ISR_NOERR 255

.extern x64_isr_handler
.type x64_isr_handler, @function

.extern CODE_SEGMENT64
.type CODE_SEGMENT64, @object


.global isr_common
.type isr_common, @function
isr_common:
        push %rax
        push %rbx
        push %rcx
        push %rdx
        push %rsi
        push %rdi
        push %rbp
        push %r8
        push %r9
        push %r10
        push %r11
        push %r12
        push %r13
        push %r14
        push %r15
        
        xor     %rax, %rax
        mov     %ds, %ax
        push    %rax

        movw    CODE_SEGMENT64(%rip), %ax
        movw    %ax, %ds
        movw    %ax, %es
        movw    %ax, %fs
        movw    %ax, %gs

        mov     %rsp, %rdi
        call    *x64_isr_handler@GOTPCREL(%rip)

        pop     %rax
        mov     %ax, %ds
        mov     %ax, %es
        mov     %ax, %fs
        mov     %ax, %gs

	pop %r15
        pop %r14
        pop %r13
        pop %r12
        pop %r11
        pop %r10
        pop %r9
        pop %r8
        pop %rbp
        pop %rdi
        pop %rsi
        pop %rdx
        pop %rcx
        pop %rbx
        pop %rax

        add     $0x10, %rsp
        iretq
.size isr_common, . - isr_common


.section .data.isr
.global isr_table
.type isr_table, @object
isr_table:
        .quad x64_ISR0
        .quad x64_ISR1
        .quad x64_ISR2
        .quad x64_ISR3
        .quad x64_ISR4
        .quad x64_ISR5
        .quad x64_ISR6
        .quad x64_ISR7
        .quad x64_ISR8
        .quad x64_ISR9
        .quad x64_ISR10
        .quad x64_ISR11
        .quad x64_ISR12
        .quad x64_ISR13
        .quad x64_ISR14
        .quad x64_ISR15
        .quad x64_ISR16
        .quad x64_ISR17
        .quad x64_ISR18
        .quad x64_ISR19
        .quad x64_ISR20
        .quad x64_ISR21
        .quad x64_ISR22
        .quad x64_ISR23
        .quad x64_ISR24
        .quad x64_ISR25
        .quad x64_ISR26
        .quad x64_ISR27
        .quad x64_ISR28
        .quad x64_ISR29
        .quad x64_ISR30
        .quad x64_ISR31
        .quad x64_ISR32
        .quad x64_ISR33
        .quad x64_ISR34
        .quad x64_ISR35
        .quad x64_ISR36
        .quad x64_ISR37
        .quad x64_ISR38
        .quad x64_ISR39
        .quad x64_ISR40
        .quad x64_ISR41
        .quad x64_ISR42
        .quad x64_ISR43
        .quad x64_ISR44
        .quad x64_ISR45
        .quad x64_ISR46
        .quad x64_ISR47
        .quad x64_ISR48
        .quad x64_ISR49
        .quad x64_ISR50
        .quad x64_ISR51
        .quad x64_ISR52
        .quad x64_ISR53
        .quad x64_ISR54
        .quad x64_ISR55
        .quad x64_ISR56
        .quad x64_ISR57
        .quad x64_ISR58
        .quad x64_ISR59
        .quad x64_ISR60
        .quad x64_ISR61
        .quad x64_ISR62
        .quad x64_ISR63
        .quad x64_ISR64
        .quad x64_ISR65
        .quad x64_ISR66
        .quad x64_ISR67
        .quad x64_ISR68
        .quad x64_ISR69
        .quad x64_ISR70
        .quad x64_ISR71
        .quad x64_ISR72
        .quad x64_ISR73
        .quad x64_ISR74
        .quad x64_ISR75
        .quad x64_ISR76
        .quad x64_ISR77
        .quad x64_ISR78
        .quad x64_ISR79
        .quad x64_ISR80
        .quad x64_ISR81
        .quad x64_ISR82
        .quad x64_ISR83
        .quad x64_ISR84
        .quad x64_ISR85
        .quad x64_ISR86
        .quad x64_ISR87
        .quad x64_ISR88
        .quad x64_ISR89
        .quad x64_ISR90
        .quad x64_ISR91
        .quad x64_ISR92
        .quad x64_ISR93
        .quad x64_ISR94
        .quad x64_ISR95
        .quad x64_ISR96
        .quad x64_ISR97
        .quad x64_ISR98
        .quad x64_ISR99
        .quad x64_ISR100
        .quad x64_ISR101
        .quad x64_ISR102
        .quad x64_ISR103
        .quad x64_ISR104
        .quad x64_ISR105
        .quad x64_ISR106
        .quad x64_ISR107
        .quad x64_ISR108
        .quad x64_ISR109
        .quad x64_ISR110
        .quad x64_ISR111
        .quad x64_ISR112
        .quad x64_ISR113
        .quad x64_ISR114
        .quad x64_ISR115
        .quad x64_ISR116
        .quad x64_ISR117
        .quad x64_ISR118
        .quad x64_ISR119
        .quad x64_ISR120
        .quad x64_ISR121
        .quad x64_ISR122
        .quad x64_ISR123
        .quad x64_ISR124
        .quad x64_ISR125
        .quad x64_ISR126
        .quad x64_ISR127
        .quad x64_ISR128
        .quad x64_ISR129
        .quad x64_ISR130
        .quad x64_ISR131
        .quad x64_ISR132
        .quad x64_ISR133
        .quad x64_ISR134
        .quad x64_ISR135
        .quad x64_ISR136
        .quad x64_ISR137
        .quad x64_ISR138
        .quad x64_ISR139
        .quad x64_ISR140
        .quad x64_ISR141
        .quad x64_ISR142
        .quad x64_ISR143
        .quad x64_ISR144
        .quad x64_ISR145
        .quad x64_ISR146
        .quad x64_ISR147
        .quad x64_ISR148
        .quad x64_ISR149
        .quad x64_ISR150
        .quad x64_ISR151
        .quad x64_ISR152
        .quad x64_ISR153
        .quad x64_ISR154
        .quad x64_ISR155
        .quad x64_ISR156
        .quad x64_ISR157
        .quad x64_ISR158
        .quad x64_ISR159
        .quad x64_ISR160
        .quad x64_ISR161
        .quad x64_ISR162
        .quad x64_ISR163
        .quad x64_ISR164
        .quad x64_ISR165
        .quad x64_ISR166
        .quad x64_ISR167
        .quad x64_ISR168
        .quad x64_ISR169
        .quad x64_ISR170
        .quad x64_ISR171
        .quad x64_ISR172
        .quad x64_ISR173
        .quad x64_ISR174
        .quad x64_ISR175
        .quad x64_ISR176
        .quad x64_ISR177
        .quad x64_ISR178
        .quad x64_ISR179
        .quad x64_ISR180
        .quad x64_ISR181
        .quad x64_ISR182
        .quad x64_ISR183
        .quad x64_ISR184
        .quad x64_ISR185
        .quad x64_ISR186
        .quad x64_ISR187
        .quad x64_ISR188
        .quad x64_ISR189
        .quad x64_ISR190
        .quad x64_ISR191
        .quad x64_ISR192
        .quad x64_ISR193
        .quad x64_ISR194
        .quad x64_ISR195
        .quad x64_ISR196
        .quad x64_ISR197
        .quad x64_ISR198
        .quad x64_ISR199
        .quad x64_ISR200
        .quad x64_ISR201
        .quad x64_ISR202
        .quad x64_ISR203
        .quad x64_ISR204
        .quad x64_ISR205
        .quad x64_ISR206
        .quad x64_ISR207
        .quad x64_ISR208
        .quad x64_ISR209
        .quad x64_ISR210
        .quad x64_ISR211
        .quad x64_ISR212
        .quad x64_ISR213
        .quad x64_ISR214
        .quad x64_ISR215
        .quad x64_ISR216
        .quad x64_ISR217
        .quad x64_ISR218
        .quad x64_ISR219
        .quad x64_ISR220
        .quad x64_ISR221
        .quad x64_ISR222
        .quad x64_ISR223
        .quad x64_ISR224
        .quad x64_ISR225
        .quad x64_ISR226
        .quad x64_ISR227
        .quad x64_ISR228
        .quad x64_ISR229
        .quad x64_ISR230
        .quad x64_ISR231
        .quad x64_ISR232
        .quad x64_ISR233
        .quad x64_ISR234
        .quad x64_ISR235
        .quad x64_ISR236
        .quad x64_ISR237
        .quad x64_ISR238
        .quad x64_ISR239
        .quad x64_ISR240
        .quad x64_ISR241
        .quad x64_ISR242
        .quad x64_ISR243
        .quad x64_ISR244
        .quad x64_ISR245
        .quad x64_ISR246
        .quad x64_ISR247
        .quad x64_ISR248
        .quad x64_ISR249
        .quad x64_ISR250
        .quad x64_ISR251
        .quad x64_ISR252
        .quad x64_ISR253
        .quad x64_ISR254
        .quad x64_ISR255
.size isr_table, . - isr_table


