//! Switching between different address description modes
//! [VA <=> PA <=> VPN <=> PPN]

use core::{fmt::{self, Debug, Formatter};
use super::PageTableEntry;

use crate::config::{
    PA_WIDTH_SV39,
    VA_WIDTH_SV39,
    PPN_WIDTH_SV39,
    VPN_WIDTH_SV39,
    PAGE_SIZE, PAGE_SIZE_BITS,
};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}

//T/usize can be From/Into each other
/* 
    T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
    T -> usize: T.0
    usize -> T:
        *1 usize.into()
        *2 T::from(_: size)

    -----------------------
    VA (39 bits)
    [VPN][PAGE_OFFSET]
    38~12/11~0

    PA (56 bits)
    [PPN][PAGE_OFFSET] 
    55~12/11~0
*/
impl From<usize> for PhysAddr {
    fn from(x: usize) -> Self {
        Self(
            x & ((1 << PA_WIDTH_SV39) - 1)
        )
    }
}

impl From<usize> for VirtAddr {
    fn from(x: usize) -> Self {
        Self (
            x & ((1 << VA_WIDTH_SV39) - 1)
        )
    }
}

impl From<usize> for PhysPageNum {
    fn from(x: usize) -> Self {
        Self (
            x & ((1 << PPN_WIDTH_SV39) - 1)
        )
    }
}

impl From<usize> for VirtPageNum {
    fn from(x: usize) -> Self {
        Self (
            x & ((1 << VPN_WIDTH_SV39) - 1)
        )
    }
}

impl From<PhysAddr> for usize {
    fn from(x: PhysAddr) -> Self {
        x.0
    }
}

impl From<VirtAddr> for usize {
    /* 
        SV39 defines [63:39] == bit[38]
        * high 256GiB: bit[38] == 1
        * low 256GiB: bit[38] == 0
        other middle va is illegal !!!
    */
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

//VPN/VA and PPN/PA can be From/Into each other
//Notice: the two are completely corresponding !!!
impl From<PhysAddr> for PhysPageNum {
    fn from(x: PhysAddr) -> Self {
        assert_eq!(x.page_offset(), 0);
        x.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(x: PhysPageNum) -> Self {
        Self(x.0 << PAGE_SIZE_BITS)
    }
}


impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    
    pub fn ceil(&self) -> PhysPageNum {
        /*
            If the PA is aligned to a page,
            the corresponding page number is taken directly,
            otherwise it is rounded up !!!
        */
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
        }
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    
    pub fn ceil(&self) -> VirtPageNum {
        /*
            If the VA is aligned to a page,
            the corresponding page number is taken directly,
            otherwise it is rounded up !!!
        */
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
        }
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut u8,
                4096
            )
        }
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTableEntry,
                4096 
            )
        }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }
}

impl VirtPageNum {
    //27bits
    pub fn cut_into_three_parts(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut vpn_idxs = [0usize; 3];
        for i in (0..3).rev() {
            vpn_idxs[i] = vpn & 511;
            vpn >>= 9;
        }
        vpn_idxs
    }
}

/* a simple range of type T */
#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: Copy + Debug + PartialEq + PartialOrd,
{
    l: T,
    r: T,
}

impl<T> SimpleRange<T>
where
    T: Copy + Debug + PartialEq + PartialOrd,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start_{:?} > end_{:?}", start, end);
        Self { l: start, r: end }
    }

    pub fn start(&self) -> T {
        self.l
    }

    pub fn end(&self) -> T {
        self.r
    }
}

pub type VPNRange = SimpleRange<VirtPageNum>;

