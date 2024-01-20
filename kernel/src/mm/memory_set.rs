//! Implementation of [MapArea/MemorySet]
/*!
    MapArea: Logical segments that make up the address space
             of the virtual program
    MemorySet: Describes the virtual address space abstraction
               of the program
 */
use crate::config::TRAMPOLINE;

use super::page_table::PTEFlags;
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

    pub fn map(&mut self, page_table: &mut PageTable) {
        
    }
    
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        
    }

}

//MemorySet: address space abstraction
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    //return `satp` of MemorySet
    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    //4KB [__alltraps/__restore]
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysADDR::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X
        );
    }

    /*
        [High-256GiB]
        * trampoline
        * app_x_kstack/guard_page
        [Low-256Gib]
        * avail_memory
        * .bss/.data/.rodata/.text
    */
    pub fn new_kernel() -> Self {
        let mut memory_set = MemorySet::new_bare();
        
        //map trampoline
        memory_set.map_trampoline();

        //TODO: map kernel sections
    }

    /*
        [High-256GiB]
        * trampoline
        * trap_context
        [Low-256Gib]
        * user_stack/guard_page
        * .bss/.data/.rodata/.text
    */
    pub fn from_elf(app_data: &[u8]) -> Self {
        
    }
    
}