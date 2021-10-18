use crate::mem::ptable::{Flag, Virtual, Physical};
use crate::proc::proc::{Process};
use crate::mem::alloc::{self};
use super::pointer::Pointer;
use std::vec::Vec;

pub struct Simulator {
    proc_list: Vec<Process>,
    curr_proc: usize,
}

impl Simulator {
    pub fn begin() -> Self {
        alloc::kinit();
        let mut v = Vec::<Process>::new();
        v.push(Process::new(0));
        Self {
            proc_list: v,
            curr_proc: 0
        }
    }

    pub fn register(&mut self, addr: Pointer<ValueType>) {
        if !self.check_valid(addr) {
            let va = Virtual::new(addr.vaddr(), addr.as_ptr() as usize);
            let paddr = alloc::kalloc().unwrap();
            let pa = Physical::new(paddr as u32, paddr as usize);
            self.proc_list[self.curr_proc].map(
                va, pa,
                &[Flag::User, Flag::Writable]
            );
        } else {
            println!("Mapping already registered for 0x{:x}.", addr.vaddr());
        }
    }

    fn check_valid(&self, addr: Pointer<ValueType>) -> bool {
        self.proc_list[self.curr_proc].mapped(
            Virtual::new(addr.vaddr(), addr.as_ptr() as usize)
        )
    }

    pub fn write(&mut self, mut addr: Pointer<ValueType>, value: ValueType) {
        match self.check_valid(addr) {
            false => println!("Invalid address 0x{:x}", addr.vaddr()),
            true => {
                // page faults...
                *addr = value
            }
        }
    }

    pub fn read(&self, addr: Pointer<ValueType>) -> Option<ValueType> {
        match self.check_valid(addr) {
            false => {
                println!("Invalid address 0x{:x}", addr.vaddr());
                None
            }
            true => Some(*addr)
        }
    }

    // pub fn fork(&self) {

    // }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValueType {
    UnsignedInt(usize),
    SignedInt(isize),
}

impl ValueType {
    pub fn get_value(&self) -> usize {
        match *self {
            Self::UnsignedInt(uint) => uint,
            Self::SignedInt(sint) => sint as usize,
        }
    }
}