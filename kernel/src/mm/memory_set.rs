//! Implementation of [MapArea/MemorySet]
/*!
    MapArea: Logical segments that make up the address space
             of the virtual program
    MemorySet: Describes the virtual address space abstraction
               of the program
 */
use crate::config::{
    TRAMPOLINE,
    PAGE_SIZE,
    MEMORY_END,
    MMIO,
    TRAP_CONTEXT,
    USER_STACK_SIZE,
};
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
use crate::sync::UPSafeCell;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::*;
use core::arch::asm;
use riscv::register::satp;


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

/* kernel address space init */
lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(
            unsafe {
                UPSafeCell::new(MemorySet::new_kernel())
            }
        );
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

/*
    MapArea: logical segment
    * ErCore specifies that each logical segment
      is mapped in the same way and with the same permissions
*/
pub struct MapArea {
    vpn_range: VPNRange,
    /*
        Only the physical page frames allocated by frame_allocater
        need to be maintained(MapType::Framed), all other mapping
        methods can find the ppn via vpn.
    */
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
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
    
    #[allow(unused)]
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let self.map_type == MapType::Framed {
            self.data_frames.remove(vpn);
        }
        page_table.unmap(vpn);
    }

    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
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

    //enable MMU
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    //install MapType::Framed area
    pub install_framed_area(&mut self, start_va: VirtAddr, 
                            end_va: VirtAddr, map_perm: MapPermission)
    {
        let area_kstack = MapArea::new(
            start_va, end_va,
            MapType::Framed, map_perm  
        );
        self.install_area(area_kstack, None);
    }

    //install MapArea
    fn install_area(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data {
            area.copy_data(&mut self.page_table, data);
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

        Note: The application kernel stack should not be allocated
        when rCore initializes the memory module, it is normally
        allocated when a new TaskControlBlock is created.
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
        memory_set.install_area(area_text, None);

        println!("[new_kernel] mapping .rodata section");
        let area_rodata = MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,  
        );
        memory_set.install_area(area_rodata, None);

        println!("[new_kernel] mapping .data section");
        let area_data = MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,  
        );
        memory_set.install_area(area_data, None);
        
        println!("[new_kernel] mapping .bss section");
        let area_bss = MapArea::new(
            (sbss as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,  
        );
        memory_set.install_area(area_rodata, None);

        println!("[new_kernel] mapping avail physical memory");
        let area_apm = MapArea::new(
            (ekernel as usize).into(),
            (MEMORY_END as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,  
        );
        memory_set.install_area(area_apm, None);

        println!("[new_kernel] mapping MMIO memory");
        for mmio in MMIO {
            let area_mmio = map_one::new(
                ((*mmio).0 as usize).into(),
                (((*mmio).0 + (*mmio).1) as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W  
            );
            memory_set.install_area(area_mmio, None);
        }
        
        //All logical sections of the kernel have been installed
        memory_set
    }

    /*
        [High-256GiB]
        * trampoline
        * trap_context
        [Low-256Gib]
        * user_stack/guard_page
        * .bss/.data/.rodata/.text

        Return (memory_set, user_stack_top, elf.entry_point)
    */
    pub fn from_elf(app_data: &[u8]) -> (Self, usize, usize) {
        let memory_set = MemorySet::new_bare();
        
        //map trampoline
        memory_set.map_trampoline();

        //map trap_context
        let area_trap_ctx = MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );
        
        //map the {.text/.rodata/.data/.bss} of application by ELF headers
        //need to carry U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.end();
                memory_set.install_area(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        
        //map user stack and guard page
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let area_stack = MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        memory_set.install_area(area_stack, None);

        //used in `sbrk`
        let area_sbrk = MapArea::new(
            user_stack_top.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        memory_set.install_area(area_sbrk, None);
        
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}