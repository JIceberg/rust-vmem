use std::vec::Vec;

static mut MEM: Memory = Memory {
    free_list: Vec::new(),
    zero_page: Page {
        data: [0; 4096],
        _ppn: 0
    }
};

#[derive(Copy, Clone)]
pub struct Page {
    data: [u8; 4096],
    _ppn: u32,
}

impl Page {
    pub(crate) fn new(ppn: u32) -> Self {
        Self {
            data: [0; 4096],
            _ppn: ppn
        }
    }

    pub fn read<T>(&self, index: usize) -> &[u8] {
        let size = std::mem::size_of::<T>();
        let s = index - index % size;
        let e = s + size;

        &self.data[s..e]
    }

    pub fn write<T>(&mut self, index: usize, data: &[u8]) {
        let size = std::mem::size_of::<T>();
        let s = index - index % size;
        let e = s + size;
        let mut i = 0;
        for idx in s..e {
            if i < data.len() {
                self.data[idx] = data[i];
                i += 1;
            }
        }
    }

    fn zero(&mut self) {
        for i in 0..4096 {
            self.write::<u8>(i, &[0]);
        }
    }
}

struct Memory {
    free_list: Vec<Page>,
    zero_page: Page,
}

impl Memory {
    fn new() -> Self {
        let mut v: Vec<Page> = Vec::new();
        for ppn in 1..32 {
            v.push(Page::new(ppn));
        }
        Self {
            free_list: v,
            zero_page: Page::new(0),
        }
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop()
    }

    fn get_zero_ref(&self) -> &Page {
        &self.zero_page
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
            Some(mut page) => {
                page.zero();
                let mut boxed = Box::new(page);
                Some(boxed.as_mut() as *mut Page)
            },
            None => None
        }
    }
}

pub fn zero_page() -> &'static Page {
    let zero: &Page;
    zero = unsafe { MEM.get_zero_ref() };
    zero
}