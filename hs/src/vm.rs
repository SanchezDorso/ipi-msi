// use core::arch::asm;
// use crate::aplic;
// use crate::aplic::aplic_init;
// use crate::console;
use crate::imsic::*;
use crate::TRAP_FRAMES;
use core::ptr::write_volatile;
// static mut
#[no_mangle]
pub fn vm_main(hartid: usize) -> (){
    // println!("Primary CPU{} enter VS mod ",hartid);
    csr_write!("sscratch", &TRAP_FRAMES[hartid]);
    imsic_init();
    let mut hartid2:usize = 0;
    if hartid == 0 {
        hartid2 = 1;
    }
    else {
        hartid2 = 0;
    }
    unsafe {
        // We are required to write only 32 bits.
        write_volatile(imsic_vs(hartid2) as *mut u32, 1);
    }
    crate::abort()
    // console::run();
}
pub fn vm2_main(hartid2:usize) -> (){
    println!("\n Secondary CPU {} enter VS mod ",hartid2);
    csr_write!("sscratch", &TRAP_FRAMES[hartid2]);
    imsic_init();
    crate::abort();
}
