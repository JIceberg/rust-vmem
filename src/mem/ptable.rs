#![allow(dead_code, unused)]

pub type PTE = PageTableEntry;
pub type PageTable = [PTE; 1024];

pub enum Flag {
    Present,
    Writable,
    User,
    WriteThrough,
    CacheDisable,
    Accessed,
    Dirty,
    Available,
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(u32);
impl PageTableEntry {
    pub(crate) fn new(ppn: u32) -> Self {
        Self(ppn << 12)
    }

    pub fn get(&self, flag: Flag) -> u32 {
        match flag {
            Flag::Present => self.0 & 1,
            Flag::Writable => (self.0 >> 1) & 1,
            Flag::User => (self.0 >> 2) & 1,
            Flag::WriteThrough => (self.0 >> 3) & 1,
            Flag::CacheDisable => (self.0 >> 4) & 1,
            Flag::Accessed => (self.0 >> 5) & 1,
            Flag::Dirty => (self.0 >> 6) & 1,
            Flag::Available => (self.0 >> 9) & 7
        }
    }

    pub fn set(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 |= 1,
            Flag::Writable => self.0 |= 1 << 1,
            Flag::User => self.0 |= 1 << 2,
            Flag::WriteThrough => self.0 |= 1 << 3,
            Flag::CacheDisable => self.0 |= 1 << 4,
            Flag::Accessed => self.0 |= 1 << 5,
            Flag::Dirty => self.0 |= 1 << 6,
            Flag::Available => self.0 |= 1 << 9
        }
    }

    pub fn clear(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 &= !(1u32),
            Flag::Writable => self.0 &= !(1u32 << 1),
            Flag::User => self.0 &= !(1u32 << 2),
            Flag::WriteThrough => self.0 &= !(1u32 << 3),
            Flag::CacheDisable => self.0 &= !(1u32 << 4),
            Flag::Accessed => self.0 &= !(1u32 << 5),
            Flag::Dirty => self.0 &= !(1u32 << 6),
            Flag::Available => self.0 &= !(1u32 << 9)
        }
    }

    pub fn get_ppn(&self) -> u32 {
        self.0 >> 12
    }
}

impl From<u32> for PageTableEntry {
    fn from(num: u32) -> Self {
        Self(num)
    }
}