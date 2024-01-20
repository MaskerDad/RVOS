//! Implementation of PageTbale_SV39
//! [PageTableEntry/PageTable]

use super::{
    frame_alloc, FrameTracker,
    PhysPageNum,
};

use bitflags::*;
use alloc::vec::Vec;
use alloc::vec;

//PTEFlags
bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

/*  
    `PageTableEntry`
    [63:54]: Reserved
    [53:28]: PPN[2]
    [27:19]: PPN[1]
    [18:10]: PPN[0]
    [9:8]:   RSW
    [7:0]:   D_A_G_U_X_W_R_V
*/
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
}

/* 
    PageTable Walking:
    `Input`
    [VA]:  VPN2_VPN1_VPN0_Offset
    [PT]:  L2_L1_L0
    [Satp]: L2_ppn

    `Output`
    PA = ((L2_ppn[VPN2])[VPN1])[VPN0] + Offset
*/
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],    
        }
    }
}