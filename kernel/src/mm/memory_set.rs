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

    /*
        A mapping is established for a page according to the mapping
        method corresponding to the logical section.All pages within
        the same logical segment are mapped in the same way.

        [MapType::Identical]
        To ensure that cpu memory access to the kernel is consistent
        before and after MMU is enabled, we need to establish an identity
        map for the kernel address space (except for TRAMPOLINE).
        [MapType::Framed]
        The virtual page number is dynamically mapped onto a physical page
        frame by means of the physical page frame allocator, which is typically
        used to apply address Spaces.
    */
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let mut ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    //Establish mappings for multiple virtual pages of a logical segment
    pub fn map(&mut self, page_table: &mut PageTable) {
        //Iterators are implemented for vpn_range
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    
    pub fn unmap_one(&mut self, page_table: &mut PageTable) {
        
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

    //install MapArea
    fn install_area(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data {
            //area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(area);
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
        * .text/.rodata/.data/.bss
    */
    pub fn new_kernel() -> Self {
        let mut memory_set = MemorySet::new_bare();
        
        //map trampoline
        memory_set.map_trampoline();

        //map kernel sections
        println!("[new_kernel] .text: [{:#x}, {:#x})", 
                stext as usize, etext as usize);
        println!("[new_kernel] .rodata: [{:#x}, {:#x})", 
                srodata as usize, erodata as usize);
        println!("[new_kernel] .data: [{:#x}, {:#x})", 
                sdata as usize, edata as usize);
        println!("[new_kernel] .bss: [{:#x}, {:#x})", 
                sbss as usize, ebss as usize);
        
        println!("[new_kernel] mapping .text section");
        let area_text = MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,  
        );
        //memory_set.install_area(area_text, None);

        println!("[new_kernel] mapping .rodata section");
        println!("[new_kernel] mapping .data section");
        println!("[new_kernel] mapping .bss section");

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