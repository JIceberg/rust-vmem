use crate::mem::ptable::{PTE, Flag, Virtual, Physical};
use crate::mem::alloc::{self, Page, PageData, DataSize};

pub struct Process {
    _pid: u32,
    pgdir: [PTE; 1024],
    tables: Vec<Page>,
}

impl Process {
    pub fn new(pid: u32) -> Self {
        Self {
            _pid: pid,
            pgdir: [PTE::new(0); 1024],
            tables: Vec::new()
        }
    }

    pub fn mapped(&self, vaddr: Virtual) -> bool {
        let va = vaddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let pde = self.pgdir[pdx];
        if !pde.get_flag(Flag::Present) {
            return false;
        }

        let pgtab = self.tables[pde.get_ppn()];
        
        let pte = PTE::from(pgtab.read(ptx, DataSize::Word).get());
        pte.get_flag(Flag::Present)
    }

    pub fn map(&mut self, vaddr: Virtual, paddr: Physical, flags: &[Flag]) -> isize {
        let va = vaddr.get();
        let pa = paddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let mut pde = self.pgdir[pdx];
        if !pde.get_flag(Flag::Present) {
            // allocate page
            if let Some(pg) = alloc::kalloc() {
                unsafe { self.tables.push(*pg); }
                pde.set(PTE::new((self.tables.len() - 1) as u32).get_address(), &[
                    Flag::Present, Flag::Protected, Flag::Writable, Flag::User
                ]);
                self.pgdir[pdx] = pde;
            } else {
                return -1;
            }
        }

        let pgtab = &mut self.tables[pde.get_ppn()];
        let mut pte = PTE::from(pgtab.read(ptx, DataSize::Word).get());

        pte.set(pa.get_address(), flags);
        pte.set_flag(Flag::Present);
        pgtab.write(ptx, PageData::Word(pte.get()));

        return 0;
    }
}