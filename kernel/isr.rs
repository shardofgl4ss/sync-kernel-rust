use core::ffi::c_void;
use crate::idt;
use crate::idt::idt_flags;
use core::ptr::addr_of;

#[repr(C, packed)]
struct isr_registers {
    ds: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    interrupt: u64,
    err: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

pub enum x86_isr_error {
    DIV = 0,
    DEBUG = 1,         // debug
    NMI = 2,           // non-maskable interrupt
    BRKP = 3,          // breakpoint
    OVRFLW = 4,        // overflow
    BOUND = 5,         // bound range exceeded
    INV_OP = 6,        // invalid opcode
    DEV_NA = 7,        // device not available
    DFAULT = 8,        // ERR: double fault
    OVERRUN = 9,       // coprocessor segment overrun (reserved)
    INV_TSS = 10,      // ERR: invalid TSS
    NO_SEG = 11,       // ERR: no segment
    STACK = 12,        // ERR: stack fault
    GP = 13,           // ERR: general protection fault
    PAGE_FAULT = 14,   // ERR: page fault
    INTEL = 15,        // reserved by intel
    FPUERR = 16,       // x87 FPU error
    ALIGNCHK = 17,     // ERR: alignment chk
    MACHINECHK = 18,   // machine chk
    AVX_FPE = 19,      // SIMD/AVX floating point exception
    VIRT = 20,         // virtualization error
    CPE = 21,          // control protection exception
    HYPER_INJECT = 28, // hypervisor injection
    VM_COMM = 29,      // ERR: VMM comm
    SECURITY = 30,     // security exception
}

unsafe extern "C" {
    pub unsafe static isr_table: [unsafe extern "C" fn(); 256];
    static CODE_SEGMENT64: u8;
}

mod handlers {
    use crate::panic::kernel_panic;

    pub fn unhandled_interrupt() {
        kernel_panic("unhandled interrupt!");
    }
    pub fn ISR_div() {
        // kputs("division by zero error!\n");
    }
    pub fn ISR_debug() {
        // kputs("debug interrupt\n");
    }
    pub fn ISR_nmi() {
        // kputs("received NMI\n");
    }
    pub fn ISR_brkp() {
        // kputs("debug breakpoint\n");
    }
    pub fn ISR_ovrflw() {
        // kputs("overflow error!\n");
    }
    pub fn ISR_bound() {
        // kputs("bound range error!\n");
    }
    pub fn ISR_inv_op() {
        // kputs("invalid opcode error!\n");
    }
    pub fn ISR_dev_na() {
        // kputs("device not available!\n");
    }
    pub fn ISR_dfault() {
        // kputs("double fault!!!\n");
        kernel_panic("double fault!");
    }
    pub fn ISR_overrun() {
        // kputs("coprocessor overrun error!\n");
    }
    pub fn ISR_inv_tss() {
        // kputs("invalid TSS error!\n");
    }
    pub fn ISR_no_seg() {
        // kputs("no segment error!\n");
    }
    pub fn ISR_stack() {
        // kputs("stack error!\n");
    }
    pub fn ISR_gp() {
        // kputs("general purpose fault error!\n");
    }
    pub fn ISR_page_fault() {
        // kputs("page fault error!\n");
    }
    pub fn ISR_intel() {
        // kputs("mfw intel\n");
    }
    pub fn ISR_fpuerr() {
        // kputs("floating point error!\n");
    }
    pub fn ISR_alignchk() {
        // kputs("alignment check error!\n");
    }
    pub fn ISR_machinechk() {
        // kputs("machine check!\n");
    }
    pub fn ISR_avx_fpe() {
        // kputs("SSE/AVX floating point exception!\n");
    }
    pub fn ISR_virt() {
        // kputs("virtualization exception!\n");
    }
    pub fn ISR_cpe() {
        // kputs("control protection exception!\n");
    }
    pub fn ISR_hyper_inject() {
        // kputs("hypervsior injection\n");
    }
    pub fn ISR_vm_comm() {
        // kputs("VM comm error!\n");
    }
    pub fn ISR_security() {
        // kputs("security exception!\n");
    }
}
static ISR_HANDLER_TABLE: [unsafe fn(); 32] = [
    handlers::ISR_div,
    handlers::ISR_debug,
    handlers::ISR_nmi,
    handlers::ISR_brkp,
    handlers::ISR_ovrflw,
    handlers::ISR_bound,
    handlers::ISR_inv_op,
    handlers::ISR_dev_na,
    handlers::ISR_dfault,
    handlers::ISR_overrun,
    handlers::ISR_inv_tss,
    handlers::ISR_no_seg,
    handlers::ISR_stack,
    handlers::ISR_gp,
    handlers::ISR_page_fault,
    handlers::ISR_intel,
    handlers::ISR_fpuerr,
    handlers::ISR_alignchk,
    handlers::ISR_machinechk,
    handlers::ISR_avx_fpe,
    handlers::ISR_virt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::unhandled_interrupt,
    handlers::ISR_cpe,
    handlers::ISR_hyper_inject,
    handlers::ISR_vm_comm,
    handlers::ISR_security
];

#[allow(clippy::match_single_binding)]
fn vector_switch_idtflags(interrupt: u8) -> u8 {
    match interrupt {
        _ => (idt_flags::F_IDT_RING_0 as u8) | (idt_flags::F_IDT_GATE_IRQ as u8),
    }
}

pub fn init() {
    unsafe {
        let cs_addr = addr_of!(CODE_SEGMENT64) as usize;

        for i in 0..255 {
            idt::setgate(
                i,
                isr_table[i as usize] as *const c_void,
                cs_addr as u16,
                vector_switch_idtflags(i),
            );
            idt::enablegate(i);
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn x64_isr_handler(frame: *const isr_registers) {
    let frame: Option<&isr_registers> = unsafe { frame.as_ref() };

    if let Some(_x) = frame { handlers::unhandled_interrupt() }
}


