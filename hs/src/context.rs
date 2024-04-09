/// Trap Context
#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 35],
    pub hsstatus: usize,
    /// CSR sstatus      
    pub sstatus: usize,
    /// CSR sepc
    pub sepc: usize,
}
