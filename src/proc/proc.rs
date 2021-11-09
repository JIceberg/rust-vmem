use crate::mem::ptable::{PTE, Flag, Virtual, Physical, Address};
use crate::mem::alloc::{self, Page};
use crate::sim::check::{ValueType, DataType};

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefMut, RefCell};

#[derive(PartialEq, Eq)]
enum ProcessState {
    Running,
    Terminated,
    Sleeping,
}

pub struct Process {
    pid: u32,
    state: ProcessState,
    pgdir: Rc<RefCell<Page>>,
    tables: Vec<Rc<RefCell<Page>>>,
    phys_pages: HashMap<u32, Rc<RefCell<Page>>>,
    debug: bool,
}

impl Process {  
    pub fn new(pid: u32, debug: bool) -> Self {
        let pgdir = match alloc::kalloc() {
            Some(dir) => dir,
            None => panic!("Out of memory")
        };
        Self {
            pid: pid,
            state: ProcessState::Sleeping,
            pgdir: Rc::new(RefCell::new(*pgdir)),
            tables: Vec::new(),
            phys_pages: HashMap::new(),
            debug,
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

        // free all physical pages
        for page_ref in self.phys_pages.values() {
            let mut page: RefMut<_> = page_ref.borrow_mut();
            if page.ref_count() == 0 {
                continue;
            }
            if page.ref_count() > 1 {
                page.decrement_refs();
            } else {
                alloc::kfree(&*page);
            }
        }
        self.phys_pages.clear();

        // free the directory
        let pgdir_ref = self.pgdir.borrow();
        alloc::kfree(&*pgdir_ref);
        for page in &self.tables {
            let page_ref = page.borrow();
            alloc::kfree(&*page_ref);
        }
        self.tables.clear();
    }

    pub fn mapped(&self, vaddr: Virtual) -> bool {
        match self.walk(vaddr) {
            Some(_) => true,
            None => false
        }
    }

    pub fn map(&mut self, vaddr: Virtual, paddr: Physical, flags: &[Flag]) {
        let mut d = self.pgdir.borrow_mut();

        let va = vaddr.get();
        let pa = paddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let raw_pd_data = d.read::<u32>(pdx * 4);
        let mut pde = PTE::from(raw_to_u32(raw_pd_data));
        if !pde.get_flag(Flag::Present) {
            // allocate page
            if let Some(pg) = alloc::kalloc() {
                pde.set(PTE::new(self.tables.len() as u32).get_address(), &[
                    Flag::Present, Flag::Protected, Flag::Writable, Flag::Accessed
                ]);
                d.write::<u32>(pdx * 4, u32_to_raw(pde.get()).as_ref());
                self.tables.push(Rc::new(RefCell::new(*pg)));
            } else {
                // eventually replace with page replacement call,
                // then panic only if that fails
                panic!("Out of memory");
            }
        }

        let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();
        let raw_data = pgtab.read::<u32>(ptx);
        let mut pte = PTE::from(raw_to_u32(raw_data));

        pte.set(pa.get_address(), flags);
        pte.set_flag(Flag::Present);
        pte.set_flag(Flag::Accessed);
        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
    }

    fn walk(&self, vaddr: Virtual) -> Option<PTE> {
        let d = self.pgdir.borrow();

        let va = vaddr.get();

        // page walk
        let pdx = va.get_dir_index();
        let ptx = va.get_table_index();
        
        let raw_pd_data = d.read::<u32>(pdx * 4);
        let pde = PTE::from(raw_to_u32(raw_pd_data));
        if !pde.get_flag(Flag::Present) {
            return None;
        }

        let pgtab = self.tables[pde.get_ppn()].borrow();
        
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
        let d = self.pgdir.borrow();
        match self.walk(vaddr) {
            Some(mut pte) => {
                let va = vaddr.get();
                if !pte.get_flag(Flag::User) {
                    panic!("Attempt to write to kernel page in user process");
                }
                
                if pte.get_flag(Flag::Zero) {
                    // lazy alloc
                    if let Some(pg) = alloc::kalloc() {
                        pte.set(PTE::new(pg.ppn()).get_address(), &[
                            Flag::Present, Flag::Writable, Flag::User
                        ]);

                        let pdx = va.get_dir_index();
                        let ptx = va.get_table_index();
                        let raw_pd_data = d.read::<u32>(pdx * 4);
                        let pde = PTE::from(raw_to_u32(raw_pd_data));
                        let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();
                        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());

                        if self.debug {
                            let pgnum = va.translate().get_address() & !0xFFF;
                            println!("PGZERO: 0x{:x}", pgnum);
                        }

                        self.phys_pages.insert(va.get_address(), Rc::new(RefCell::new(*pg)));

                        drop(d);
                        drop(pgtab);

                       self.write(vaddr, value);
                       return;
                    } else {
                        // eventually replace with page replacement call,
                        // then panic only if that fails
                        panic!("Out of memory");
                    }
                } else {
                    if pte.get_flag(Flag::Writable) {
                        if let Some(page) = self.phys_pages.get(&va.get_address()) {
                            let mut page_ref = page.borrow_mut();
                            match value {
                                ValueType::SignedInt(_) =>
                                    page_ref.write::<isize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                                ValueType::UnsignedInt(_) =>
                                    page_ref.write::<usize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                                ValueType::Zero =>
                                    page_ref.write::<usize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                            }
                            let pdx = va.get_dir_index();
                            let ptx = va.get_table_index();
                            let raw_pd_data = d.read::<u32>(pdx * 4);
                            let pde = PTE::from(raw_to_u32(raw_pd_data));
                            let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();

                            pte.set_flag(Flag::Dirty);
                            pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());

                            drop(pgtab);
                            drop(d);
                            return;
                        }
                    } else {
                        // attempting to write to non-writable page
                        // we can assume this means that the page is a copy or copied for CoW

                        // obtain the page
                        if let Some(old_page) = self.phys_pages.get_mut(&va.get_address()) {
                            let mut old_pg = old_page.borrow_mut();

                            // check ref count
                            if old_pg.ref_count() > 1 {
                                // there are processes still referencing this page
                                old_pg.decrement_refs();
                                if let Some(pg) = alloc::kalloc() {
                                    pg.copy(&*old_pg);
                                    drop(old_pg);
                                    
                                    pte.set(PTE::new(pg.ppn()).get_address(), &[
                                        Flag::Present, Flag::Writable, Flag::User
                                    ]);
            
                                    let pdx = va.get_dir_index();
                                    let ptx = va.get_table_index();
                                    let raw_pd_data = d.read::<u32>(pdx * 4);
                                    let pde = PTE::from(raw_to_u32(raw_pd_data));
                                    {
                                        let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();
                                        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
                                    }
            
                                    if self.debug {
                                        let pgnum = va.translate().get_address() & !0xFFF;
                                        println!("PGCOPY: 0x{:x}", pgnum);
                                    }
            
                                    self.phys_pages.insert(va.get_address(), Rc::new(RefCell::new(*pg)));
                                    
                                    drop(d);
                                    self.write(vaddr, value);
                                    return;
                                }
                            } else {
                                // there are no other processes referencing this page,
                                // so simply mark it as writable and retry
                                drop(old_pg);
                                let pdx = va.get_dir_index();
                                let ptx = va.get_table_index();
                                let raw_pd_data = d.read::<u32>(pdx * 4);
                                let pde = PTE::from(raw_to_u32(raw_pd_data));
                                {
                                    drop(d);
                                    let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();

                                    pte.set_flag(Flag::Writable);
                                    pte.clear_flag(Flag::Dirty);
                                    pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
                                }
                                self.write(vaddr, value);
                                return;
                            }
                        }
                    }
                }
            },
            None => println!("Invalid address 0x{:x}", vaddr.get().get_address())
        }
    }

    pub fn read(&self, vaddr: Virtual, data_type: DataType) -> Option<ValueType> {
        if self.state != ProcessState::Running {
            println!("ZOMBIE {}", self.pid);
            return None;
        }
        match self.walk(vaddr) {
            Some(pte) => {
                let va = vaddr.get();
                if pte.get_flag(Flag::Zero) {
                    return Some(ValueType::UnsignedInt(0));
                }
                if let Some(page) = self.phys_pages.get(&va.get_address()) {
                    let my_page = page.borrow();
                    let val = match data_type {
                        DataType::SignedInt => {
                            let data = my_page.read::<isize>(va.get_offset() as usize);
                            let mut num = 0;
                            let mut shif = 0;
                            for &byte in data {
                                num |= (byte as usize) << shif;
                                shif += 8;
                            }
                            ValueType::SignedInt(num as isize)
                        }
                        DataType::UnsignedInt => {
                            let data = my_page.read::<usize>(va.get_offset() as usize);
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

    pub fn copy(&mut self, child_pid: u32, debug: bool) -> Self {
        let mut pages = HashMap::new();
        for (&key, page) in self.phys_pages.iter_mut() {
            let mut pg = page.borrow_mut();
            let va = Address::Virtual(key, 0);

            // page walk to ensure proper mapping
            let pdx = va.get_dir_index();
            let ptx = va.get_table_index();
            
            let d = self.pgdir.borrow();
            let raw_pd_data = d.read::<u32>(pdx * 4);
            let pde = PTE::from(raw_to_u32(raw_pd_data));
            if !pde.get_flag(Flag::Present) {
                panic!("PTE does not exist");
            }

            let mut pgtab = self.tables[pde.get_ppn()].borrow_mut();
            let raw_data = pgtab.read::<u32>(ptx * 4);
            let mut pte = PTE::from(raw_to_u32(raw_data));
            if !pte.get_flag(Flag::Present) {
                panic!("Page not present");
            }

            if pte.get_flag(Flag::Zero) {
                pages.insert(0, Rc::new(RefCell::new(*alloc::zero_page())));
            } else {
                // increase ref count
                pg.increment_refs();

                // no longer writable
                pte.clear_flag(Flag::Writable);
                pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());
                
                pages.insert(va.get_address(), Rc::new(RefCell::new(*pg)));
            }
        }

        // copy tables
        let mut tables: Vec<Rc<RefCell<Page>>> = Vec::new();
        for table in self.tables.iter_mut() {
            let page = match alloc::kalloc() {
                Some(tb) => tb,
                None => panic!("Out of memory")
            };
            page.copy(&*table.borrow());
            tables.push(Rc::new(RefCell::new(*page)));
        }

        // copy pgdir
        let pgdir = match alloc::kalloc() {
            Some(dir) => dir,
            None => panic!("Out of memory")
        };
        pgdir.copy(&*self.pgdir.borrow());

        self.yieldk();
        Self {
            pid: child_pid,
            state: ProcessState::Sleeping,
            pgdir: Rc::new(RefCell::new(*pgdir)),
            tables,
            phys_pages: pages,
            debug,
        }
    }

    pub fn print_mem(&self) {
        if self.debug {
            println!("PAGE DIRECTORY\n");
            for i in 0..1024 {
                let d = self.pgdir.borrow();
                let raw_pd_data = d.read::<u32>(i * 4);
                let entry = PTE::from(raw_to_u32(raw_pd_data));
                println!("PDE #{}\t PTN: {}, Flags: 0x{:x}", i, entry.get_ppn(), entry.get() & 0xFFF);
            }
            println!();
            for i in 0..self.tables.len() {
                println!("PAGE TABLE #{}\n", i);
                for j in 0..1024 {
                    let table = self.tables[i].borrow();
                    let entry = PTE::from(raw_to_u32(table.read::<u32>(j * 4)));
                    println!("PTE #{}\t PPN: {}, Flags: 0x{:x}", j, entry.get_ppn(), entry.get() & 0xFFF);
                }
                println!();
            }
            println!();
            for (_, page) in self.phys_pages.iter() {
                let pg = page.borrow();
                println!("PAGE #{}\n", pg.ppn());
                for i in 0..1024 {
                    let word = raw_to_u32(pg.read::<u32>(i * 4));
                    println!("Word #{}: 0x{:x}", i, word);
                }
                println!();
            }
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