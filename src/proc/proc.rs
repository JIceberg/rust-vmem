use crate::mem::ptable::{PTE, Flag, Virtual, Physical, Address};
use crate::mem::alloc::{self, Page};
use crate::sim::check::{ValueType, DataType};
use crate::sim::pointer::Pointer;

use std::collections::HashMap;

pub struct Process {
    _pid: u32,
    pgdir: [PTE; 1024],
    tables: Vec<Page>,
    phys_pages: HashMap<usize, Page>,
}

impl Process {  
    pub fn new(pid: u32) -> Self {
        Self {
            _pid: pid,
            pgdir: [PTE::new(0); 1024],
            tables: Vec::new(),
            phys_pages: HashMap::new()
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
        
        let pde = self.pgdir[pdx];
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
        match self.walk(vaddr) {
            Some(mut pte) => {
                let va = vaddr.get();
                if pte.get_flag(Flag::Zero) {
                    // lazy alloc
                    if let Some(pg) = alloc::kalloc() {
                        let ptr_pg = Pointer::<Page>::from(pg as usize);
                        let pg_vaddr = Address::Virtual(ptr_pg.vaddr(), pg as usize);
                        pte.set(pg_vaddr.translate().get_address(), &[
                            Flag::Present, Flag::Writable, Flag::User
                        ]);

                        let pdx = va.get_dir_index();
                        let ptx = va.get_table_index();
                        let pde = self.pgdir[pdx];
                        let pgtab = &mut self.tables[pde.get_ppn()];
                        pgtab.write::<u32>(ptx * 4, u32_to_raw(pte.get()).as_ref());

                        {
                            let pgnum = vaddr.get().translate().get_address() & !0xFFF;
                            println!("PGZERO: 0x{:x}", pgnum);
                        }

                        self.phys_pages.insert(pte.get_ppn(), unsafe {*pg});

                       self.write(vaddr, value);
                    } else {
                        // eventually replace with page replacement call,
                        // then panic only if that fails
                        panic!("Out of memory");
                    }
                } else {
                    if let Some(page) = self.phys_pages.get_mut(&pte.get_ppn()) {
                        match value {
                            ValueType::SignedInt(_) =>
                                page.write::<isize>(va.get_offset() as usize, value.as_bytes().as_ref()),
                            ValueType::UnsignedInt(_) =>
                                page.write::<usize>(va.get_offset() as usize, value.as_bytes().as_ref()),
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
                if let Some(page) = self.phys_pages.get(&pte.get_ppn()) {
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

    pub fn print_mem(&self) {
        println!("PAGE DIRECTORY\n");
        for i in 0..self.pgdir.len() {
            let entry = self.pgdir[i];
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
        for page_num in self.phys_pages.keys() {
            println!("PAGE #{}\n", page_num);
            let page = self.phys_pages.get(&page_num).unwrap();
            for i in 0..1024 {
                let word = raw_to_u32(page.read::<u32>(i * 4));
                println!("Word #{}: 0x{:x}", i, word);
            }
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