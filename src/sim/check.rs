use crate::mem::ptable::{Flag, Virtual, Physical};
use crate::proc::proc::{Process};
use crate::mem::alloc::{self, Page};
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

    pub fn register<T>(&mut self, addr: Pointer<T>) {
        if !self.check_valid(addr) {
            let va = Virtual::new(addr.vaddr(), addr.as_ptr() as usize);
            let pg = alloc::zero_page();
            let pa = Physical::new(pg as *const Page as u32,
                                   pg as *const Page as usize);
            self.proc_list[self.curr_proc].map(
                va, pa,
                &[Flag::User, Flag::Zero]
            );
        } else {
            println!("Mapping already registered for 0x{:x}.", addr.vaddr());
        }
    }

    fn check_valid<T>(&self, addr: Pointer<T>) -> bool {
        self.proc_list[self.curr_proc].mapped(
            Virtual::new(addr.vaddr(), addr.as_ptr() as usize)
        )
    }

    pub fn write<T>(&mut self, addr: Pointer<T>, value: ValueType) {
        let proc = &mut self.proc_list[self.curr_proc];
        proc.write(Virtual::new(addr.vaddr(), addr.as_ptr() as usize), value);
    }

    pub fn read(&self, addr: Pointer<ValueType>, data_type: DataType) -> Option<ValueType> {
        let proc = &self.proc_list[self.curr_proc];
        proc.read(Virtual::new(addr.vaddr(), addr.as_ptr() as usize), data_type)
    }

    pub fn print(&self) {
        self.proc_list[self.curr_proc].print_mem();
    }

    // pub fn fork(&self) {

    // }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValueType {
    UnsignedInt(usize),
    SignedInt(isize),
}

pub enum DataType {
    SignedInt,
    UnsignedInt
}

impl ValueType {
    pub fn get_value(&self) -> usize {
        match *self {
            Self::UnsignedInt(uint) => uint,
            Self::SignedInt(sint) => sint as usize,
        }
    }

    pub fn as_bytes(&self) -> Box<[u8]> {
        match *self {
            Self::UnsignedInt(uint) => {
                let mut v: Vec<u8> = Vec::new();

                let mut shif = 0;
                for _ in 0..std::mem::size_of::<usize>() {
                    v.push(((uint >> shif) & 0xFF) as u8);
                    shif += 8;
                }

                v.into_boxed_slice()
            }
            Self::SignedInt(sint) => {
                let mut v: Vec<u8> = Vec::new();

                let mut shif = 0;
                for _ in 0..std::mem::size_of::<usize>() {
                    v.push(((sint >> shif) & 0xFF) as u8);
                    shif += 8;
                }

                v.into_boxed_slice()
            }
        }
    }
}