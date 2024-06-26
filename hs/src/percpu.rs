use crate::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use core::sync::atomic::Ordering;
use crate::vm_main::secondary_vm_main;
pub struct ArchCpu {
    pub x: [usize; 32],
    pub hartid: usize,
}

impl ArchCpu {
    pub fn new(hartid: usize) -> Self {
        ArchCpu {
            x: [0; 32],
            hartid,
        }
    }
    pub fn get_hartid(&self) -> usize {
        self.hartid
    }
    pub fn init(&mut self, cpu_id: usize) -> usize {
        csr_write!("sscratch", self as *const _ as usize);
        self.hartid = cpu_id; //cpu id
        0
    }
    pub fn run_boot(&mut self) {
        let hartid: usize = self.hartid;
        unsafe {
            core::arch::asm!("
                mv a0, {0}", 
                in(reg) hartid,
            );
            core::arch::asm!("sret");
        }
    }
    pub fn run_noboot(&mut self) {
        let hartid2: usize = self.hartid;
        unsafe {
            let secondary_vm_main_ptr: fn(usize) = secondary_vm_main;
            let secondary_vm_main_addr = secondary_vm_main_ptr as *const fn(usize) as usize; 
            core::arch::asm!("mv t0, {0}", in(reg) secondary_vm_main_addr);
            core::arch::asm!("csrw sepc ,t0");
            core::arch::asm!("
            mv a0, {0}", 
            in(reg) hartid2,
        );
            core::arch::asm!("sret");
        }
    }
}

pub struct PerCpu {
    pub id: usize,
    pub arch_cpu: ArchCpu,
    pub boot_cpu: bool,
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'a mut Self {
        let _cpu_rank = crate::ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
        let paddr = PER_CPU_ARRAY_PTR.wrapping_add(cpu_id * PER_CPU_SIZE); // 使用物理地址
        let ret = unsafe { &mut *(paddr as *mut Self) };
        *ret = PerCpu {
            id: cpu_id,
            arch_cpu: ArchCpu::new(cpu_id),
            boot_cpu: false,
        };
        ret
    }

    pub fn cpu_init(&mut self) {
        self.arch_cpu.init(self.id);
    }
    pub fn run_vm(&mut self) {
        let self_id = self.id;
        // println!("prepare CPU{} for vm run!", self_id);
        if self.boot_cpu {
            println!("boot vm on CPU{}!", self_id);
            self.arch_cpu.run_boot();
        } else {
            // crate::imsic::imsic_init();
            // unsafe {
            //     core::arch::asm!("wfi");
            // }
            self.arch_cpu.run_noboot();
        }
    }
    
    pub fn set_boot(&mut self){
        self.boot_cpu = true;
    }
}