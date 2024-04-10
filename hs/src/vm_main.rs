use crate::imsic::*;
use crate::TRAP_FRAMES;
// static mut
#[no_mangle]
pub fn primary_vm_main(hartid: usize) -> (){
    // println!("Primary CPU{} enter VS mod ",hartid);
    csr_write!("sscratch", &TRAP_FRAMES[0]);
    unsafe {
        TRAP_FRAMES[0][35] = hartid;  
        TRAP_FRAMES[0][34] = 1;  
    }  
    imsic_init();
    imsic_ipi_trigger(crate::another_hartid(hartid));
    crate::abort()
}
pub fn secondary_vm_main(hartid2:usize) -> (){
    println!("\nSecondary CPU {} enter VS mod ",hartid2);
    csr_write!("sscratch", &TRAP_FRAMES[1]);
    unsafe {
        TRAP_FRAMES[1][35] = hartid2;  
        TRAP_FRAMES[0][34] = 0;  
    }  
    imsic_init();
    crate::abort();
}
