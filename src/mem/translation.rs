use super::ptable::{PageTable, PTE};

pub enum Address {
    Virtual(u32),
    Physical(u32),
}

impl Address {
    pub fn translate(&self, dir: &PageTable, tables: &[PageTable; 1024]) -> Self {
        match *self {
            Self::Virtual(vaddr) => {
                let dir_index = (vaddr >> 22) & 0x3FF;
                let table_index = (vaddr >> 12) & 0x3FF;
                let offset = vaddr & 0x3FF;
                
                let dir_ppn = (dir[dir_index as usize] >> 12) & 0xFFFFF;
                let table = tables[dir_ppn as usize];
                let entry: PTE = table[table_index as usize];
                
                let table_ppn = (entry >> 12) & 0xFFFFF;
                let paddr = (table_ppn << 12) | offset;

                Self::Physical(paddr)
            }
            Self::Physical(paddr) => {
                Address::Virtual(paddr)
            }
        }
    }

    pub fn get_address(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr) => vaddr,
            Self::Physical(paddr) => paddr
        }
    }
}