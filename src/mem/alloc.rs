use std::vec::Vec;

static mut MEM: Memory = Memory {
    free_list: Vec::new()
};

#[derive(Copy, Clone)]
pub struct Page {
    data: [u8; 4096],
    _ppn: u32,
}

pub enum DataSize {
    Byte,
    Word,
}

pub enum PageData {
    Byte(u8),
    Word(u32),
}

impl PageData {
    pub fn get(&self) -> u32 {
        match *self {
            Self::Byte(byte) => byte as u32,
            Self::Word(word) => word
        }
    }
}

impl Page {
    pub(crate) fn new(ppn: u32) -> Self {
        Self {
            data: [0; 4096],
            _ppn: ppn
        }
    }

    pub fn read(&self, index: usize, size: DataSize) -> PageData {
        match size {
            DataSize::Byte => PageData::Byte(self.data[index]),
            DataSize::Word => {
                let idx = index - index % 4;
                let word: u32 = self.data[idx] as u32
                         | ((self.data[idx+1] as u32) << 8)
                         | ((self.data[idx+2] as u32) << 16)
                         | ((self.data[idx+3] as u32) << 24);
                PageData::Word(word)
            }
        }
    }

    pub fn write(&mut self, index: usize, data: PageData) {
        match data {
            PageData::Byte(byte) => self.data[index] = byte,
            PageData::Word(word) => {
                let idx = index - index % 4;
                self.data[idx] = (word & 0xFF) as u8;
                self.data[idx+1] = ((word >> 8) & 0xFF) as u8;
                self.data[idx+2] = ((word >> 16) & 0xFF) as u8;
                self.data[idx+3] = ((word >> 24) & 0xFF) as u8;
            }
        }
    }
}

struct Memory {
    free_list: Vec<Page>,
}

impl Memory {
    fn new() -> Self {
        let mut v: Vec<Page> = Vec::new();
        for ppn in 0..16 {
            v.push(Page::new(ppn));
        }
        Self {
            free_list: v
        }
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop()
    }
}

pub fn kinit() {
    unsafe {
        MEM = Memory::new()
    }
}

pub fn kalloc() -> Option<*mut Page> {
    unsafe {
        match MEM.pop_free() {
            Some(page) => {
                let mut boxed = Box::new(page);
                Some(boxed.as_mut() as *mut Page)
            }
            None => None
        }
    }
}