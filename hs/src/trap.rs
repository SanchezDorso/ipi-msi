use crate::imsic::{imsic_handle, PrivMode};
// use riscv::register::mtvec::TrapMode;
// use riscv::register::{stvec,vstvec};
// use core::arch:: global_asm;
// use core::arch::asm;
use crate::percpu::*;
// use crate::println;
// global_asm!(include_str!("trap.S"));
// extern "C" {
//     fn trap();
// }
// pub fn init() {
//     unsafe {
//         // Set the trap vector.
//         stvec::write(trap as usize, TrapMode::Direct);
//     }
// }


#[no_mangle]
pub fn rust_trap() {
    let scause = csr_read!("scause");
    let interrupt = scause >> 63 & 1 == 1;
    println!("scause: 0x{:08x}",scause);
    // let hartid = current_cpu.hartid;
    // println!("hartid {}",hartid);
    if interrupt {
        // Interrupt (asynchronous)
        match scause & 0xFF {
            9 => imsic_handle(PrivMode::Supervisor),
            // 11 => imsic_handle(PrivMode::Machine),
            _ => println!("Unknown interrupt #{}", scause),
        }
    } else {
        match scause & 0xFF {
            //10 => Exception::VirtualSupervisorEnvCall,
            10 => println!("ecall to S mod "),
            _ =>  panic!("Unknown exception ")
        }
    }
}
