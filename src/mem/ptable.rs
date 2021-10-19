#![allow(dead_code, unused)]

pub type PTE = PageTableEntry;

pub const KERNBASE: u32 = 0x80000000;
pub const PDXSHIFT: usize = 22;
pub const PTXSHIFT: usize = 12;
pub const PAGESIZE: usize = 4096;

#[derive(Copy, Clone)]
pub enum Flag {
    Present,
    Writable,
    User,
    WriteThrough,
    CacheDisable,
    Accessed,
    Dirty,
    Protected,
    Zero,
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub(crate) fn new(ppn: u32) -> Self {
        Self(ppn << PTXSHIFT & !0xFFF)
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Present => self.0 & 1 == 1,
            Flag::Writable => (self.0 >> 1) & 1 == 1,
            Flag::User => (self.0 >> 2) & 1 == 1,
            Flag::WriteThrough => (self.0 >> 3) & 1 == 1,
            Flag::CacheDisable => (self.0 >> 4) & 1 == 1,
            Flag::Accessed => (self.0 >> 5) & 1 == 1,
            Flag::Dirty => (self.0 >> 6) & 1 == 1,
            Flag::Protected => (self.0 >> 7) & 1 == 1,
            Flag::Zero => (self.0 >> 8) & 1 == 1,
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 |= 1,
            Flag::Writable => self.0 |= 1 << 1,
            Flag::User => self.0 |= 1 << 2,
            Flag::WriteThrough => self.0 |= 1 << 3,
            Flag::CacheDisable => self.0 |= 1 << 4,
            Flag::Accessed => self.0 |= 1 << 5,
            Flag::Dirty => self.0 |= 1 << 6,
            Flag::Protected => self.0 |= 1 << 7,
            Flag::Zero => self.0 |= 1 << 8,
        }
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 &= !(1u32),
            Flag::Writable => self.0 &= !(1u32 << 1),
            Flag::User => self.0 &= !(1u32 << 2),
            Flag::WriteThrough => self.0 &= !(1u32 << 3),
            Flag::CacheDisable => self.0 &= !(1u32 << 4),
            Flag::Accessed => self.0 &= !(1u32 << 5),
            Flag::Dirty => self.0 &= !(1u32 << 6),
            Flag::Protected => self.0 &= !(1u32 << 7),
            Flag::Zero => self.0 &= !(1u32 << 8),
        }
    }

    pub fn get_ppn(&self) -> usize {
        (self.0 >> PTXSHIFT) as usize
    }

    pub fn set(&mut self, pa: u32, flags: &[Flag]) {
        self.0 = pa & !0xFFFu32;
        for &flag in flags {
            self.set_flag(flag);
        }
    }

    pub fn get_address(&self) -> u32 {
        self.0 & !0xFFFu32
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<u32> for PageTableEntry {
    fn from(num: u32) -> Self {
        Self(num)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Address {
    Virtual(u32, usize),
    Physical(u32, usize),
}

#[derive(Copy, Clone)]
pub struct Virtual(Address);
impl Virtual {
    pub fn new(vaddr: u32, ptr: usize) -> Self {
        Self(Address::Virtual(vaddr, ptr))
    }

    pub fn get(&self) -> Address {
        self.0
    }

    pub fn as_phys(&self) -> Physical {
        Physical::new(
            self.0.translate().get_address(),
            self.0.get_ptr() as usize
        )
    }
}

#[derive(Copy, Clone)]
pub struct Physical(Address);
impl Physical {
    pub fn new(paddr: u32, ptr: usize) -> Self {
        Self(Address::Physical(paddr, ptr))
    }

    pub fn get(&self) -> Address {
        self.0
    }

    pub fn from(ppn: u32, offset: u32, ptr: usize) -> Self {
        let paddr = ppn | offset;
        Self(Address::Physical(paddr, ptr))
    }
}

impl Address {
    pub fn translate(&self) -> Self {
        match *self {
            Self::Virtual(vaddr, ptr) => Self::Physical(vaddr - KERNBASE, ptr),
            Self::Physical(paddr, ptr) => Self::Virtual(paddr + KERNBASE, ptr)
        }
    }

    pub fn get_address(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr, _) => vaddr,
            Self::Physical(paddr, _) => paddr
        }
    }

    pub fn get_dir_index(&self) -> usize {
        match *self {
            Self::Virtual(vaddr, _) => ((vaddr >> PDXSHIFT) & 0x3FF) as usize,
            Self::Physical(_, _) => 0,
        }
    }

    pub fn get_table_index(&self) -> usize {
        match *self {
            Self::Virtual(vaddr, _) => ((vaddr >> PTXSHIFT) & 0x3FF) as usize,
            Self::Physical(_, _) => 0,
        }
    }

    pub fn get_ptr(&self) -> *mut u32 {
        match *self {
            Self::Virtual(_, ptr) => ptr as *mut u32,
            Self::Physical(_, ptr) => ptr as *mut u32
        }
    }

    pub fn get_offset(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr, _) => vaddr & 0xFFF,
            Self::Physical(paddr, _) => paddr & 0xFFF,
        }
    }
}

use core::ops::{Deref, DerefMut};

impl Deref for Address {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        unsafe { & *self.get_ptr() }
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.get_ptr() }
    }
}