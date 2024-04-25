
use spin::RwLock;
use spin::Once;

const APLIC_S: usize = 0xd00_0000;
// S-mode interrupt delivery controller
const APLIC_S_IDC: usize = 0xd00_4000;
pub const APLIC_DOMAINCFG_BASE: usize = 0x0000;
pub const APLIC_SOURCECFG_BASE: usize = 0x0004;
pub const APLIC_MSIAADDR_BASE: usize = 0x1BC8;
pub const APLIC_PENDING_BASE: usize = 0x1C00;
pub const APLIC_CLRIP_BASE: usize = 0x1D00;
pub const APLIC_ENABLE_BASE: usize = 0x1E00;
pub const APLIC_CLRIE_BASE: usize = 0x1F00;
pub const APLIC_TARGET_BASE: usize = 0x3004;

#[repr(u32)]
#[allow(dead_code)]
pub enum SourceModes { 
    Inactive = 0,
    Detached = 1,
    RisingEdge = 4,
    FallingEdge = 5,
    LevelHigh = 6,
    LevelLow = 7,
}
pub static APLIC: Once<RwLock<Aplic>> = Once::new();

pub fn host_aplic<'a>() -> &'a RwLock<Aplic> {
    APLIC.get().expect("Uninitialized hypervisor aplic!")
}
pub fn init_aplic(aplic_base: usize, aplic_size: usize) {
    let aplic = Aplic::new(aplic_base, aplic_size);
    APLIC.call_once(|| RwLock::new(aplic));
}
// offset size register name
// 0x0000 4 bytes domaincfg
// 0x0004 4 bytes sourcecfg[1]
// 0x0008 4 bytes sourcecfg[2]
// . . . . . .
// 0x0FFC 4 bytes sourcecfg[1023]
// 0x1BC0 4 bytes mmsiaddrcfg (machine-level interrupt domains only)
// 0x1BC4 4 bytes mmsiaddrcfgh ”
// 0x1BC8 4 bytes smsiaddrcfg ”
// 0x1BCC 4 bytes smsiaddrcfgh ”
// 0x1C00 4 bytes setip[0]
// 0x1C04 4 bytes setip[1]
// . . . . . .
// 0x1C7C 4 bytes setip[31]
// 0x1CDC 4 bytes setipnum
// 0x1D00 4 bytes in clrip[0]
// 0x1D04 4 bytes in clrip[1]
// . . . . . .
// 0x1D7C 4 bytes in clrip[31]
// 0x1DDC 4 bytes clripnum
// 0x1E00 4 bytes setie[0]
// 0x1E04 4 bytes setie[1]
// . . . . . .
// 0x1E7C 4 bytes setie[31]
// 0x1EDC 4 bytes setienum
// 0x1F00 4 bytes clrie[0]
// 0x1F04 4 bytes clrie[1]
// . . . . . .
// 0x1F7C 4 bytes clrie[31]
// 0x1FDC 4 bytes clrienum
// 0x2000 4 bytes setipnum le
// 0x2004 4 bytes setipnum be
// 0x3000 4 bytes genmsi
// 0x3004 4 bytes target[1]
// 0x3008 4 bytes target[2]
// . . . . . .
// 0x3FFC 4 bytes target[1023]


#[repr(C)]
pub struct Aplic {
    pub base: usize,
    pub size: usize,
}

#[allow(dead_code)]
impl Aplic {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
        }
    }
    pub fn set_domaincfg(&self, bigendian: bool, msimode: bool, enabled: bool){
        let enabled = u32::from(enabled);
        let msimode = u32::from(msimode);
        let bigendian = u32::from(bigendian);
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let src = (enabled << 8) | (msimode << 2) | bigendian;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn set_sourcecfg(&self, irq: u32, mode: SourceModes){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = mode as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn set_sourcecfg_delegate(&self, irq: u32, child: u32){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = 1 << 10 | child & 0x3ff;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn set_msiaddr(&self, address: usize){
        let addr = self.base + APLIC_MSIAADDR_BASE;
        let src = (address >> 12) as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
            core:: ptr::write_volatile((addr + 4) as *mut u32, 0);
        }
    }
    pub fn read_pending(&self, irq: u32) -> u32{
        assert!(irq > 0 && irq < 1024);
        let irqidx = irq as usize / 32;
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_pending(&self, irq: u32, pending: bool){
        assert!(irq > 0 && irq < 1024);
        let irqidx = irq as usize / 32;
        let irqbit = irq as usize % 32;
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        let src = 1 << irqbit;
        if pending {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, src);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, src);
            }
        }
    } 
    pub fn read_enable(&self, irq: u32) -> u32{
        assert!(irq > 0 && irq < 1024);
        let irqidx = irq as usize / 32;
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    // pub fn set_enable(&self, irq: u32, enabled: bool){
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let irqbit = irq as usize % 32;
    //     let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
    //     let clr_addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
    //     let src = 1 << irqbit;
    //     if enabled {
    //         unsafe{
    //             core:: ptr::write_volatile(addr as *mut u32, src);
    //         }
    //     } else {
    //         unsafe{
    //             core:: ptr::write_volatile(clr_addr as *mut u32, src);
    //         }
    //     }
    // } 
    pub fn set_enable(&self, irqidx: usize, value: u32, enabled: bool){
        assert!(irqidx < 32);
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        println!("clr_addr {:0x}", clr_addr);
        println!("value {:0x}", value);
        if enabled {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, value);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, value);
            }
        }
    } 
    pub fn set_target_msi(&self, irq: u32, hart: u32, guest: u32, eiid: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src = (hart << 18) | (guest << 12) | eiid;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn set_target_direct(&self, irq: u32, hart: u32, prio: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src =  (hart << 18) | (prio & 0xFF);
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
}

#[repr(C)]
struct InterruptDeliveryControl {
    pub idelivery: u32,
    pub iforce: u32,
    pub ithreshold: u32,
    pub topi: u32,
    pub claimi: u32,
}

#[allow(dead_code)]
impl InterruptDeliveryControl {
    const fn ptr(hart: usize) -> *mut Self {
        assert!(hart < 1024);
        (APLIC_S_IDC + hart * 32) as *mut Self
    }

    pub fn as_ref<'a>(hart: usize) -> &'a Self {
        unsafe { Self::ptr(hart).as_ref().unwrap() }
    }

    pub fn as_mut<'a>(hart: usize) -> &'a mut Self {
        unsafe { Self::ptr(hart).as_mut().unwrap() }
    }
}

pub fn aplic_init(hartid:usize) {
    init_aplic(APLIC_S, 0x8000);
    let hartid_u32: u32 = hartid as u32;
    let host_aplic = host_aplic();
    host_aplic.write().set_enable(0, 0xffffffff, false);
    // host_aplic.write().set_domaincfg(false, true, true);
    // host_aplic.write().set_sourcecfg(10, SourceModes::LevelHigh);
    // host_aplic.write().set_enable(10, true);
    // host_aplic.write().set_target_msi(10, hartid_u32, 1, 10);
}
