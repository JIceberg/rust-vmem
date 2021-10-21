#![allow(dead_code, unused)]

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
        let mut proc = Process::new(0);
        proc.wake_up();
        v.push(proc);
        Self {
            proc_list: v,
            curr_proc: 0
        }
    }

    pub fn register<T>(&mut self, addr: Pointer<T>) {
        if !self.check_valid(addr) {
            let va = Virtual::new(addr.vaddr(), addr.as_ptr() as usize);
            let pg = alloc::zero_page();
            let pa = Physical::new(0, pg as *const Page as usize);
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

    pub fn read<T>(&self, addr: Pointer<T>, data_type: DataType) -> Option<ValueType> {
        let proc = &self.proc_list[self.curr_proc];
        proc.read(Virtual::new(addr.vaddr(), addr.as_ptr() as usize), data_type)
    }

    pub fn fork(&mut self) {
        let size = self.proc_list.len() as u32;
        let mut new_proc = self.proc_list[self.curr_proc].copy(size);
        new_proc.wake_up();
        self.curr_proc = new_proc.pid() as usize;
        self.proc_list.push(new_proc);
    }

    pub fn kill(&mut self) {
        self.proc_list[self.curr_proc].kill();
        self.proc_list.remove(self.curr_proc);
        if self.curr_proc > 0 {
            self.curr_proc -= 1;
            self.proc_list[self.curr_proc].wake_up();
        }
    }

    pub fn switch(&mut self, proc_num: usize) {
        self.proc_list[self.curr_proc].yieldk();
        self.curr_proc = proc_num;
        self.proc_list[self.curr_proc].wake_up();
    }

    pub fn print(&self) {
        for proc in self.proc_list.iter() {
            println!("PROCESS PID {}\n", proc.pid());
            proc.print_mem();
            println!();
        }
    }
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