//! Implementation of [MapArea/MemorySet]
/*!
    MapArea: Logical segments that make up the address space
             of the virtual program
    MemorySet: Describes the virtual address space abstraction
               of the program
 */
use super::{
    VirtPageNum,
    VirtAddr,
    PhysPageNum,
    PhysAddr,
};
use super::{
    frame_alloc,
    FrameTracker,  
};
use super::{
    VPNRange,
};
use super::{
    PageTable,
};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

 extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
 }

/*
    MapArea: logical segment
    * ErCore specifies that each logical segment
      is mapped in the same way and with the same permissions
*/
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
}

//MemorySet: address space abstraction
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    
}