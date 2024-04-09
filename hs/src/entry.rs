//     .section .text.entry
//     .globl _start
// _start:
//     la sp, boot_stack_top
    
//     # 1 << 7 is SPV   2<<32 is VSXL  1 <<12 is VGEIN
//     li      t0, (1 << 7) | (2 << 32) | (1 << 12)
//     csrw    hstatus, t0

//     # HIDELEG_VSSI| HIDELEG_VSTI | HIDELEG_VSEI
//     li      t0, (1 << 2) | (1 << 6) | (1 << 10)
//     csrw    hie, t0
    
//     # HIDELEG_VSSI| HIDELEG_VSTI | HIDELEG_VSEI
//     li      t0, (1 << 2) | (1 << 6) | (1 << 10)
//     csrw    hideleg, t0

//     #  1 << 8 is SPP 1 << 5 is SPIE
//     li      t0, (1 << 8) | (1 << 5)
//     csrw    sstatus, t0

//     #           SSIE        STIE        SEIE
//     li      t0, (1 << 1) | (1 << 5) | (1 << 9)
//     csrw    sie, t0

//     la      t0, trap
//     csrw    stvec, t0

//     la      t0, rust_main
//     csrw    sepc, t0
    
//     #  1 << 8 is SPP 1 << 5 is SPIE
//     li      t0, (1 << 1) | (1 << 5)
//     csrw    vsstatus, t0

//     #           SSIE        STIE        SEIE
//     li      t0, (1 << 1) | (1 << 5) | (1 << 9)
//     csrw    vsie, t0

//     la      t0, trap
//     csrw    vstvec, t0
    
//     sret

//     .section .bss.stack
//     .globl boot_stack_lower_bound
// boot_stack_lower_bound:
//     .space 4096 * 16
//     .globl boot_stack_top
// boot_stack_top:
// use core::arch::global_asm; // 支持内联汇编
use crate::consts::PER_CPU_SIZE;
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn _start() -> i32 {
    //a0=cpuid,a1=dtb addr
    core::arch::asm!(
        "
        la t0,__core_end  
        li t1,{per_cpu_size}            //t1=per_cpu_size
        mul t2,a0,t1                    //t2=cpuid*per_cpu_size
        add t2,t1,t2                    //t2=cpuid*per_cpu_size+per_cpu_size
        add sp,t0,t2                    //sp=core_end+cpuid*per_cpu_size+per_cpu_size
        
        # 1 << 7 is SPV   2<<32 is VSXL  1 <<12 is VGEIN
        li      t0, (1 << 7) | (2 << 32) | (1 << 12)
        csrw    hstatus, t0
    
        # HIDELEG_VSSI| HIDELEG_VSTI | HIDELEG_VSEI
        li      t0, (1 << 2) | (1 << 6) | (1 << 10)
        csrw    hie, t0
        
        # HIDELEG_VSSI| HIDELEG_VSTI | HIDELEG_VSEI
        li      t0, (1 << 2) | (1 << 6) | (1 << 10)
        csrw    hideleg, t0
    
        #  1 << 8 is SPP 1 << 5 is SPIE
        li      t0, (1 << 8) | (1 << 5)
        csrw    sstatus, t0
    
        #           SSIE        STIE        SEIE
        li      t0, (1 << 1) | (1 << 5) | (1 << 9)
        csrw    sie, t0
    
        la      t0, trap
        csrw    stvec, t0
    
        la      t0, vm_main
        csrw    sepc, t0
        
        #  1 << 8 is SPP 1 << 5 is SPIE
        li      t0, (1 << 1) | (1 << 5)
        csrw    vsstatus, t0
    
        #           SSIE        STIE        SEIE
        li      t0, (1 << 1) | (1 << 5) | (1 << 9)
        csrw    vsie, t0
    
        la      t0, trap
        csrw    vstvec, t0

        call rust_main
        ",
        // rust_main = sym crate::rust_main,
        per_cpu_size= const PER_CPU_SIZE ,
        options(noreturn),
    );
}