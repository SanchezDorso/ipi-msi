use crate::imsic::{imsic_handle, PrivMode};
#[no_mangle]
pub fn rust_trap(trap_frames: &mut [[usize; 36]; 2]) {
    let hartid =(*trap_frames)[0][35];
    let boot_cpu: bool = (*trap_frames)[0][34] != 0;    let scause = csr_read!("scause");
    let interrupt = scause >> 63 & 1 == 1;
    println!("scause: 0x{:08x}",scause);
    if interrupt {
        // Interrupt (asynchronous)
        match scause & 0xFF {
            9 => imsic_handle(PrivMode::Supervisor,hartid, boot_cpu),
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
