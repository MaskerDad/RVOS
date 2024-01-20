//! Implementation of PageTbale_SV39
//! [PageTableEntry/PageTable]

use super::{
    frame_alloc, FrameTracker,
    PhysPageNum, VirtPageNum,
};

use bitflags::*;
use alloc::vec::Vec;
use alloc::vec;
use riscv::paging::PageTable;

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

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & (1usize << 44) - 1).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn empty() -> Self {
        Self { bits: 0usize }
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
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
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],    
        }
    }
    
    /*
        For satp:
        [63:60] => MODE
            * 0 => disable MMU
            * 8 => enable MMu
        [59:44] => ASID
        [43:0]  => PPN
    */
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }

    //real page_table walking
    pub fn find_pte_create(&mut self, vpn: VirtPageNum)
        -> Option<&mut PageTableEntry>
    {
        let vpn_idxs = vpn.cut_into_three_parts();
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTableEntry> = None;
        for (i, vpn_idx) in vpn_idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*vpn_idx];
            if (i == 2) {
                res = Some(pte);
                break;
            }
            if (!pte.is_valid()) {
                let frame_pt = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame_pt.ppn, PTEFlags::V);
                self.frames.push(frame_pt);
            }
            ppn = pte.ppn();
        }
        res
    }

    pub fn find_pte(&self, vpn: VirtPageNum)
        -> Option<&mut PageTableEntry>
    {
        let vpn_idxs = vpn.cut_into_three_parts();
        let ppn = self.root_ppn;
        let mut res: Option<&mut PageTableEntry> = None;
        for (i, vpn_idx) in vpn_idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array[*vpn_idx];
            if (i == 2) {
                res = Some(pte);
                break;
            }         
            if (!pte.is_valid()) {
                return None;
            }
            ppn = pte.ppn();
        }
        res
    }

    pub fn map(&mut self, vpn: VirtPageNum,
                ppn: PhysPageNum, flags: PTEFlags)
    {
        let pte_final = self.find_pte_create(vpn).unwrap();
        assert!(!pte_final.is_valid(), 
                "vpn {:?} is mapped before mapping!", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    //clear final pte
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte_final = self.find_pte(vpn).unwrap();
        assert!(pte_final.is_valid(),
                "vpn {:?} is invalid before unmapping", vpn);
        *pte_final = PageTableEntry::empty();
    }
}