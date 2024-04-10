#![allow(dead_code)]
use crate::console::console_irq;
use core::{arch::asm, ptr::write_volatile};

// Each hart is a page away from each other (4096 bytes or 0x1000)
const IMSIC_HART_STRIDE: usize = 0x1000;
const IMSIC_VS_HART_STRIDE: usize = 0x2000;

// There are two IMSICs per HART
//   one for machine mode (M)
//   one for supervisor mode (S)
pub const IMSIC_M: usize = 0x2400_0000;
pub const IMSIC_S: usize = 0x2800_0000;
pub const IMSIC_VS: usize = 0x2800_1000;
// Helper functions for determining MMIO address
// for the messages. Each HART has an M and S mode
// IMSIC. Each HART has its own IMSIC in its own page.
const fn imsic_m(hart: usize) -> usize {
    IMSIC_M + IMSIC_HART_STRIDE * hart
}

pub const fn imsic_s(hart: usize) -> usize {
    IMSIC_S + IMSIC_HART_STRIDE * hart
}

pub const fn imsic_vs(hart: usize) -> usize {
    IMSIC_VS + IMSIC_VS_HART_STRIDE * hart
}

// We only use XLEN for the EIE and EIP
// since there are multiple registers based on the
// interrupt number to enable or to set pending.
const XLEN: usize = usize::BITS as usize;
const XLEN_STRIDE: usize = XLEN / 32;

// The following are used as parameters to a match statement.
// However, I chose to use the same number as their CSRs so
// that if you need to cross-reference it, you have multiple
// places to look.

// M-mode IMSIC CSRs
const MISELECT: usize = 0x350;
const MIREG: usize = 0x351;
const MTOPI: usize = 0xFB0;
const MTOPEI: usize = 0x35C;

// S-Mode IMSIC CSRs
const SISELECT: usize = 0x150;
const SIREG: usize = 0x151;
const STOPI: usize = 0xDB0;
const STOPEI: usize = 0x15C;

// VS-Mode IMSIC CSRs
const VSISELECT: usize = 0x250;
const VSIREG: usize = 0x251;
const VSTOPI: usize = 0xEB0;
const VSTOPEI: usize = 0x25C;

// Constants for MISELECT/MIREG
// Pass one of these into MISELECT
// Then the MIREG will reflect that register
const EIDELIVERY: usize = 0x70;
const EITHRESHOLD: usize = 0x72;

// For 32-bit, we have 0x80 for messages 0..31
//                     0x81 for messages 32..63
// and so forth.

// For 64-bit, 0x80 covers 0x81 for messages 0..63
// Referencing 0x81 or any other odd-numbered CSR will cause
// an illegal instruction.

// Same goes for EIP and EIE
const EIP: usize = 0x80;
const EIE: usize = 0xC0;

pub enum PrivMode {
    Machine = 0,
    Supervisor = 1
}
pub enum CSR {
    SISELECT,
    SIREG,
    STOPI,
    STOPEI
}

// Currently, the CSRs for the IMSICs are not recognized by my
// assembler. Luckily, we can specify any value for the CSR. If it
// doesn't exist, we will get a trap #2 (illegal instruction).

// Write to an IMSIC CSR
fn imsic_write(reg: usize, val: usize) {
    unsafe {
        match reg {
            MISELECT => asm!("csrw 0x350, {val}", val = in(reg) val),
            SISELECT => asm!("csrw 0x150, {val}", val = in(reg) val),
            // VSISELECT => asm!("csrw 0x250, {val}", val = in(reg) val),

            MIREG => asm!("csrw 0x351, {val}", val = in(reg) val),
            SIREG => asm!("csrw 0x151, {val}", val = in(reg) val),
            // VSIREG => asm!("csrw 0x251, {val}", val = in(reg) val),

            MTOPI => asm!("csrw 0xFB0, {val}", val = in(reg) val),
            STOPI => asm!("csrw 0xDB0, {val}", val = in(reg) val),
            // VSTOPI => asm!("csrw 0xEB0, {val}", val = in(reg) val),

            MTOPEI => asm!("csrw 0x35C, {val}", val = in(reg) val),
            STOPEI => asm!("csrw 0x15C, {val}", val = in(reg) val),
            // VSTOPEI => asm!("csrw 0x25C, {val}", val = in(reg) val),

            _ => panic!("Unknown CSR {}", reg),
        }
    }
}

// Read from an IMSIC CSR
fn imsic_read(reg: usize) -> usize {
    let ret: usize;
    unsafe {
        match reg {
            MISELECT => asm!("csrr {val}, 0x350", val = out(reg) ret),
            SISELECT => asm!("csrr {val}, 0x150", val = out(reg) ret),
            // VSISELECT => asm!("csrr {val}, 0x250", val = out(reg) ret),

            MIREG => asm!("csrr {val}, 0x351", val = out(reg) ret),
            SIREG => asm!("csrr {val}, 0x151", val = out(reg) ret),
            // VSIREG => asm!("csrr {val}, 0x251", val = out(reg) ret),

            MTOPI => asm!("csrr {val}, 0xFB0", val = out(reg) ret),
            STOPI => asm!("csrr {val}, 0xDB0", val = out(reg) ret),
            // VSTOPI => asm!("csrr {val}, 0xEB0", val = out(reg) ret),

            MTOPEI => asm!("csrr {val}, 0x35C", val = out(reg) ret),
            STOPEI => asm!("csrr {val}, 0x15C", val = out(reg) ret),
            // VSTOPEI => asm!("csrr {val}, 0x25C", val = out(reg) ret),

            _ => panic!("Unknown CSR {}", reg),
        }
    }
    ret
}

// Enable a message number
fn imsic_enable(mode: PrivMode, which: usize) {
    let eiebyte = EIE + XLEN_STRIDE * which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, eiebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg | 1 << bit);
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, eiebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg | 1 << bit);
        }
        // PrivMode::VS => {
        //     imsic_write(VSISELECT, eiebyte);
        //     let reg = imsic_read(VSIREG);
        //     imsic_write(VSIREG, reg | 1 << bit);
        // }
    };
}

fn imsic_disable(mode: PrivMode, which: usize) {
    let eiebyte = EIE + XLEN_STRIDE * which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, eiebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg & !(1 << bit));
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, eiebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg & !(1 << bit));
        }
        // PrivMode::VS => {
        //     imsic_write(VSISELECT, eiebyte);
        //     let reg = imsic_read(VSIREG);
        //     imsic_write(VSIREG, reg & !(1 << bit));
        // }
    };
}
fn imsic_read_eip(which: usize){
    let eipbyte = EIP + XLEN_STRIDE * which / XLEN;
    imsic_write(SISELECT, eipbyte);
    let reg = imsic_read(SIREG);
    println!("imsic eip 0x{:08x}",reg);
}
fn imsic_trigger(mode: PrivMode, which: usize) {
    let eipbyte = EIP + XLEN_STRIDE * which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, eipbyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg | 1 << bit);
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, eipbyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg | 1 << bit);
        }
        // PrivMode::VS => {
        //     imsic_write(VSISELECT, eipbyte);
        //     let reg = imsic_read(VSIREG);
        //     imsic_write(VSIREG, reg | 1 << bit);
        // }
    };
}

pub fn imsic_ipi_trigger(hartid: usize){
    unsafe {
        // We are required to write only 32 bits.
        write_volatile(imsic_vs(hartid) as *mut u32, 1);
    }
}

fn imsic_clear(mode: PrivMode, which: usize) {
    let eipbyte = EIP + XLEN_STRIDE * which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, eipbyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg & !(1 << bit));
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, eipbyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg & !(1 << bit));
        }
        // PrivMode::VS => {
        //     imsic_write(VSISELECT, eipbyte);
        //     let reg = imsic_read(VSIREG);
        //     imsic_write(VSIREG, reg & !(1 << bit));
        // }
    };
}

pub fn imsic_init() {
    // let hartid = csr_read!("mhartid");
    // First, enable the interrupt file
    // 0 = disabled
    // 1 = enabled
    // 0x4000_0000 = use PLIC instead
    // imsic_write(MISELECT, EIDELIVERY);
    // imsic_write(MIREG, 1);
    imsic_write(SISELECT, EIDELIVERY);
    imsic_write(SIREG, 1);
    // imsic_write(VSISELECT, EIDELIVERY);
    // imsic_write(VSIREG, 1);

    // Set the interrupt threshold.
    // 0 = enable all interrupts
    // P = enable < P only
    // Priorities come from the interrupt number directly
    // imsic_write(MISELECT, EITHRESHOLD);
    // // Only hear 0, 1, 2, 3, and 4
    // imsic_write(MIREG, 5);

    // Hear message 10
    imsic_write(SISELECT, EITHRESHOLD);
    imsic_write(SIREG, 11);

    // imsic_write(VSISELECT, EITHRESHOLD);
    // imsic_write(VSIREG, 5);

    // Enable message #10. This will be UART when delegated by the
    // APLIC.
    // imsic_enable(PrivMode::Machine, 2);
    // imsic_enable(PrivMode::Machine, 4);
    imsic_enable(PrivMode::Supervisor, 1);
    imsic_enable(PrivMode::Supervisor, 2);
    imsic_enable(PrivMode::Supervisor, 4);
    imsic_enable(PrivMode::Supervisor, 10);
    // imsic_enable(PrivMode::VS, 2);
    // imsic_enable(PrivMode::VS, 4);

    // Trigger interrupt #2
    // SETEIPNUM no longer works
    // This can be done via SETEIPNUM CSR or via MMIO
    // imsic_write!(csr::s::SETEIPNUM, 2);
    // unsafe {
    //     // We are required to write only 32 bits.
    //     write_volatile(imsic_vs(hart) as *mut u32, 2)
    // }
    // imsic_trigger(PrivMode::Supervisor, 4);
}

fn imsic_pop(pr: PrivMode) -> u32 {
    let ret: u32;
    unsafe {
        match pr {
            // MTOPEI
            PrivMode::Machine => asm!("csrrw {ret}, 0x35C, zero", ret = out(reg) ret),
            // STOPEI
            PrivMode::Supervisor => asm!("csrrw {ret}, 0x15C, zero", ret = out(reg) ret),
            // PrivMode::VS => asm!("csrrw {ret}, 0x25C, zero", ret = out(reg) ret),

        }
    }
    // I originally had ret & 0x7FF, but the specification recommends ret >> 16
    ret >> 16
}

/// Handle an IMSIC trap. Called from `trap::rust_trap`
pub fn imsic_handle(pm: PrivMode, hartid: usize, boot_cpu:bool) {
    let msgnum = imsic_pop(pm);
    match msgnum {
        0 => println!("Spurious 'no' message."),
        1 => {
            if boot_cpu{
                println!("IPI from Boot CPU {} to another.\n",hartid);
            }
            else {
                println!("IPI from Boot CPU {} to another.\n",crate::another_hartid(hartid));
            }
        }
        2 => println!("IN VS-mod triggered by MMIO write successful!"),
        4 => println!("IN VS-mod triggered by EIP successful!"),
        10 => {
            console_irq();
            println!("IRQ 10 to boot CPU {}",hartid);
            imsic_ipi_trigger(crate::another_hartid(hartid));
        }
        _ => println!("Unknown msi #{}", msgnum),
    }
}
