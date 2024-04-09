//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.
#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use crate::consts::*;
use crate::percpu::PerCpu;

global_asm!(include_str!("trap.S"));

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::console::Uart, $($args)+);
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("[ABORT]: ");
    if let Some(p) = info.location() {
        println!("line {}, file {}", p.line(), p.file());
    } else {
        println!("no information available.");
    }
    abort();
}

pub fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
static mut TRAP_FRAMES: [[usize; 36]; 2] = [[0; 36]; 2];
static MASTER_CPU: AtomicI32 = AtomicI32::new(-1);
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);


fn primary_init_late() {
    INIT_LATE_OK.store(1, Ordering::Release);
}

fn wakeup_secondary_cpus(this_id: usize,dtb: usize) {
    for cpu_id in 0..consts::MAX_CPU_NUM {
        if cpu_id == this_id {
            continue;
        }
        // println!("wake up");
        // syscall::sys_wake(this_id, dtb);
        sbi_rt::hart_start(cpu_id, consts::HV_PHY_BASE,dtb);
    }
}


fn primary_init_early(cpuid:usize){
    clear_bss();
    println!("Hello, world!");
    let mut hartid2:usize = 0;
    if cpuid == 0 {
        hartid2 = 1;
    }
    else {
        hartid2 = 0;
    }
    console::uart_init();
    aplic::aplic_init(hartid2);

    INIT_EARLY_OK.store(1, Ordering::Release);
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32){
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}
fn wait_for(condition: impl Fn() -> bool){
    while condition() {
        core::hint::spin_loop();
    }
}

// Control and Status Register macros to read/write CSRs
#[macro_export]
macro_rules! csr_write {
    ($csr: expr, $val: expr) => ( unsafe {
        core::arch::asm!(concat!("csrw ", $csr, ", {value}"), value = in(reg) $val);
    })
}

#[macro_export]
macro_rules! csr_read {
    ($csr: expr) => ( unsafe {
        let ret: usize;
        core::arch::asm!(concat!("csrr {ret}, ", $csr), ret = out(reg) ret);
        ret
    })
}


/// the rust entry-point of os
#[no_mangle]
pub fn rust_main(cpuid:usize, host_dtb: usize) -> ! {
    csr_write!("sscratch", &TRAP_FRAMES[cpuid]);
    let mut is_primary = false;
    if MASTER_CPU.load(Ordering::Acquire) == -1 {
        MASTER_CPU.store(cpuid as i32, Ordering::Release);
        is_primary = true;
    }
    let cpu = PerCpu::new(cpuid);
    println!("Hello from CPU {}!", cpuid);
    if is_primary {
        cpu.set_boot();
        wakeup_secondary_cpus(cpuid as usize, host_dtb);
    }
    wait_for(|| ENTERED_CPUS.load(Ordering::Acquire) < MAX_CPU_NUM as _);
    assert_eq!(ENTERED_CPUS.load(Ordering::Acquire), MAX_CPU_NUM as _);

    if is_primary {
        println!("Primary CPU {} entered",cpuid);
        primary_init_early(cpuid); // create root cell here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1);
        println!("Secondary CPU {} entered",cpuid);
    }


    INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for_counter(&INITED_CPUS, MAX_CPU_NUM as _);
    cpu.cpu_init();

    if is_primary {
        primary_init_late();
    } else {
        wait_for_counter(&INIT_LATE_OK, 1);
    }

    cpu.run_vm();
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
pub mod aplic;
pub mod console;
pub mod imsic;
pub mod ringbuffer;
pub mod trap;
pub mod consts;
pub mod entry;
pub mod percpu;
pub mod vs_main;
