use crate::mem::ptable::{PTE, Flag, Virtual, Physical, Address};
use crate::mem::alloc::{self, Page};
use crate::sim::check::{ValueType, DataType};
use crate::sim::pointer::Pointer;

use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
enum ProcessState {
    Running,
    Terminated,
    Sleeping,
}

pub struct Process {
    pid: u32,
    state: ProcessState,
    pgdir: Page,
    tables: Vec<Page>,
    phys_pages: HashMap<u32, Page>,
    page_refs: HashMap<u32, Rc<Page>>,
}

impl Process {  
    pub fn new(pid: u32) -> Self {
        let pgdir = match alloc::kalloc() {
            Some(dir) => unsafe { *dir },
            None => panic!("Out of memory")
        };
        Self {
            pid: pid,
            state: ProcessState::Sleeping,
            pgdir: pgdir,
            tables: Vec::new(),
            phys_pages: HashMap::new(),
            page_refs: HashMap::new(),
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn wake_up(&mut self) {
        self.state = ProcessState::Running;
    }

    pub fn yieldk(&mut self) {
        self.state = ProcessState::Sleeping;
    }

    pub fn kill(&mut self) {
        self.state = ProcessState::Terminated;

        // free all physical pages owned by the process
        for page in &mut self.phys_pages.values_mut() {
            if page.ref_count() > 1 {
                page.decrement_refs();
            } else {
                alloc::kfree(page);
            }
        }

        // free all physical pages referenced by the process
        for page_rc in &mut self.page_refs.values_mut() {
            let page = Rc::make_mut(page_rc);
            if page.ref_count() > 1 {
                page.decrement_refs();
            } else {
                alloc::kfree(page);
            }
        }

        // free the directory
        alloc::kfree(&mut self.pgdir);
        for page in &mut self.tables {
            alloc::kfree(page);
        }
    }

    pub fn mapped(&self, vaddr: Virtual) -> bool {
        match self.walk(vaddr) {
            Some(_) => true,
            None => false
        }
    }

    pub fn map(&mut self, vaddr: Virtual, paddr: Physical, flags: &[Flag]) {
        let va = vaddr.get();
        let pa = paddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
        let mut pde = PTE::from(raw_to_u32(raw_pd_data));
        if !pde.get_flag(Flag::Present) {
            // allocate page
            if let Some(pg) = alloc::kalloc() {
                pde.set(PTE::new(self.tables.len() as u32).get_address(), &[
                    Flag::Present, Flag::Protected, Flag::Writable, Flag::User
                ]);
                self.pgdir.write::<u32>(pdx * 4, u32_to_raw(pde.get()).as_ref());
                unsafe { self.tables.push(*pg); }
            } else {
                // eventually replace with page replacement call,
                // then panic only if that fails
                panic!("Out of memory");
            }
        }

        let pgtab = &mut self.tables[pde.get_ppn()];
        let raw_data = pgtab.read::<u32>(ptx);
        let mut pte = PTE::from(raw_to_u32(raw_data));

        pte.set(pa.get_address(), flags);
        pte.set_flag(Flag::Present);
        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
    }

    fn walk(&self, vaddr: Virtual) -> Option<PTE> {
        let va = vaddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
        let pde = PTE::from(raw_to_u32(raw_pd_data));
        if !pde.get_flag(Flag::Present) {
            return None;
        }

        let pgtab = self.tables[pde.get_ppn()];
        
        let raw_data = pgtab.read::<u32>(ptx * 4);
        let pte = PTE::from(raw_to_u32(raw_data));
        match pte.get_flag(Flag::Present) {
            true => Some(pte),
            false => None
        }
    }

    pub fn write(&mut self, vaddr: Virtual, value: ValueType) {
        if self.state != ProcessState::Running {
            println!("ZOMBIE {}", self.pid);
            return;
        }
        match self.walk(vaddr) {
            Some(mut pte) => {
                let va = vaddr.get();
                if pte.get_flag(Flag::Zero) {
                    // lazy alloc
                    if let Some(pg) = alloc::kalloc() {
                        let ptr_pg = Pointer::<Page>::from(pg as usize);
                        let pg_vaddr = Address::Virtual(ptr_pg.vaddr(), pg as usize);
                        pte.set(pg_vaddr.translate().get_address() & !0xFFF, &[
                            Flag::Present, Flag::Writable, Flag::User
                        ]);

                        let pdx = va.get_dir_index();
                        let ptx = va.get_table_index();
                        let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
                        let pde = PTE::from(raw_to_u32(raw_pd_data));
                        let pgtab = &mut self.tables[pde.get_ppn()];
                        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());

                        {
                            let pgnum = va.translate().get_address() & !0xFFF;
                            println!("PGZERO: 0x{:x}", pgnum);
                        }

                        self.phys_pages.insert(pte.get_address(), unsafe {*pg});

                       self.write(vaddr, value);
                       return;
                    } else {
                        // eventually replace with page replacement call,
                        // then panic only if that fails
                        panic!("Out of memory");
                    }
                } else {
                    if pte.get_flag(Flag::Writable) {
                        if let Some(page) = self.phys_pages.get_mut(&pte.get_address()) {
                            match value {
                                ValueType::SignedInt(_) =>
                                    page.write::<isize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                                ValueType::UnsignedInt(_) =>
                                    page.write::<usize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                            }
                        }
                    } else {
                        // attempting to write to non-writable page
                        // we can assume this means that the page is a copy for CoW

                        // first, check if parent or child
                        if let Some(_) = self.phys_pages.get(&pte.get_address()) {
                            // it was the parent, make it writable and retry
                            let pdx = va.get_dir_index();
                            let ptx = va.get_table_index();
                            let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
                            let pde = PTE::from(raw_to_u32(raw_pd_data));
                            let pgtab = &mut self.tables[pde.get_ppn()];

                            pte.set_flag(Flag::Writable);
                            pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());

                            self.write(vaddr, value);
                            return;
                        }

                        // it was the child, so allocate new page and remove old reference
                        if let Some(old_page) = &mut self.page_refs.remove(&pte.get_address()) {
                            Rc::make_mut(old_page).decrement_refs();
                            if let Some(pg) = alloc::kalloc() {
                                let ptr_pg = Pointer::<Page>::from(pg as usize);
                                let pg_vaddr = Address::Virtual(ptr_pg.vaddr(), pg as usize);
                                pte.set(pg_vaddr.translate().get_address(), &[
                                    Flag::Present, Flag::Writable, Flag::User
                                ]);
        
                                let pdx = va.get_dir_index();
                                let ptx = va.get_table_index();
                                let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
                                let pde = PTE::from(raw_to_u32(raw_pd_data));
                                let pgtab = &mut self.tables[pde.get_ppn()];
                                pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
        
                                {
                                    let pgnum = va.translate().get_address() & !0xFFF;
                                    println!("PGCOPY: 0x{:x}", pgnum);
                                }
        
                                self.phys_pages.insert(pte.get_address(), unsafe {*pg});
        
                                self.write(vaddr, value);
                            }
                        } else {
                            // kill process
                        }
                    }
                }
            },
            None => println!("Invalid address 0x{:x}", vaddr.get().get_address())
        }
    }

    pub fn read(&self, vaddr: Virtual, data_type: DataType) -> Option<ValueType> {
        match self.walk(vaddr) {
            Some(pte) => {
                let va = vaddr.get();
                if pte.get_flag(Flag::Zero) {
                    return Some(ValueType::UnsignedInt(0));
                }
                let pg = match pte.get_flag(Flag::Writable) {
                    true => self.phys_pages.get(&pte.get_address()),
                    false => {
                        match self.page_refs.get(&pte.get_address()) {
                            Some(kref) => Some(kref.as_ref()),
                            // below is the case where the child died before the parent
                            None => self.phys_pages.get(&pte.get_address())
                        }
                    }
                };
                if let Some(page) = pg {
                    let val = match data_type {
                        DataType::SignedInt => {
                            let data = page.read::<isize>(va.get_offset() as usize);
                            let mut num = 0;
                            let mut shif = 0;
                            for &byte in data {
                                num |= (byte as usize) << shif;
                                shif += 8;
                            }
                            ValueType::SignedInt(num as isize)
                        }
                        DataType::UnsignedInt => {
                            let data = page.read::<usize>(va.get_offset() as usize);
                            let mut num = 0;
                            let mut shif = 0;
                            for &byte in data {
                                num |= (byte as usize) << shif;
                                shif += 8;
                            }
                            ValueType::UnsignedInt(num as usize)
                        }
                    };
                    return Some(val);
                }
            },
            None => println!("Invalid address 0x{:x}", vaddr.get().get_address())
        }
        None
    }

    pub fn copy(&mut self, child_pid: u32) -> Self {
        let mut refs = self.page_refs.clone();
        for (&key, page) in self.phys_pages.iter_mut() {
            let mut pte = PTE::from(key);
            let va = Address::Physical(pte.get_address(), 0).translate();

            // page walk to ensure proper mapping
            let pdx = va.get_dir_index();
            let ptx = va.get_table_index();
            
            let raw_pd_data = self.pgdir.read::<u32>(pdx * 4);
            let pde = PTE::from(raw_to_u32(raw_pd_data));
            if !pde.get_flag(Flag::Present) {
                panic!("PTE does not exist");
            }

            let pgtab = &mut self.tables[pde.get_ppn()];
            let raw_data = pgtab.read::<u32>(ptx * 4);
            pte = PTE::from(raw_to_u32(raw_data));
            if !pte.get_flag(Flag::Present) {
                panic!("Page not present");
            }

            if pte.get_flag(Flag::Zero) {
                
            } else {
                // increase ref count
                page.increment_refs();

                // no longer writable
                pte.clear_flag(Flag::Writable);
                pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
                
                refs.insert(pte.get_address(), Rc::from(*page));
            }
        }
        self.yieldk();
        Self {
            pid: child_pid,
            state: ProcessState::Sleeping,
            pgdir: self.pgdir,
            tables: self.tables.clone(),
            phys_pages: HashMap::new(),
            page_refs: refs.clone(),
        }
    }

    pub fn print_mem(&self) {
        println!("PAGE DIRECTORY\n");
        for i in 0..1024 {
            let raw_pd_data = self.pgdir.read::<u32>(i * 4);
            let entry = PTE::from(raw_to_u32(raw_pd_data));
            println!("PDE #{}\t PTN: {}, Flags: 0x{:x}", i, entry.get_ppn(), entry.get() & 0xFFF);
        }
        println!();
        for i in 0..self.tables.len() {
            println!("PAGE TABLE #{}\n", i);
            for j in 0..1024 {
                let entry = PTE::from(raw_to_u32(self.tables[i].read::<u32>(j * 4)));
                println!("PTE #{}\t PPN: {}, Flags: 0x{:x}", j, entry.get_ppn(), entry.get() & 0xFFF);
            }
            println!();
        }
        println!();
        for (key, page) in self.phys_pages.iter() {
            println!("PAGE #{}\n", key >> 12);
            for i in 0..1024 {
                let word = raw_to_u32(page.read::<u32>(i * 4));
                println!("Word #{}: 0x{:x}", i, word);
            }
            println!();
        }
        println!();
        for key in self.page_refs.keys() {
            println!("PAGE #{} (REFERENCED)\n", key >> 12);
            println!();
        }
    }
}

fn raw_to_u32(raw_data: &[u8]) -> u32 {
    let mut shif = 0;
    let mut val: u32 = 0;
    for i in 0..raw_data.len() {
        val |= (raw_data[i] as u32) << shif;
        shif += 8;
    }
    val
}

fn u32_to_raw(data: u32) -> Box<[u8]> {
    let mut v = Box::new(Vec::<u8>::new());

    let mut shif = 0;
    for _ in 0..4 {
        v.push(((data >> shif) & 0xFF) as u8);
        shif += 8;
    }

    v.into_boxed_slice()
}