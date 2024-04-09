// use core::arch::asm;
// use crate::aplic;
// use crate::aplic::aplic_init;
// use crate::console;
use crate::imsic::*;
use crate::TRAP_FRAMES;
// static mut
#[no_mangle]
pub fn primary_vs_main(hartid: usize) -> (){
    // println!("Primary CPU{} enter VS mod ",hartid);
    csr_write!("sscratch", &TRAP_FRAMES[0]);
    unsafe {
        TRAP_FRAMES[0][35] = hartid;  
    }  
    imsic_init();
    imsic_ipi_trigger(crate::another_hartid(hartid));
    crate::abort()
}
pub fn secondary_vs_main(hartid2:usize) -> (){
    println!("\nSecondary CPU {} enter VS mod ",hartid2);
    csr_write!("sscratch", &TRAP_FRAMES[1]);
    unsafe {
        TRAP_FRAMES[1][35] = hartid2;  
    }  
    imsic_init();
    crate::abort();
}
