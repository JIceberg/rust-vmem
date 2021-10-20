use std::vec::Vec;

static ZERO_PAGE: Page = Page {
    data: [0; 4096],
    ref_count: 0,
    ppn: 0
};

static mut MEM: Memory = Memory {
    free_list: Vec::new(),
    zero_page: &ZERO_PAGE
};

#[derive(Copy, Clone)]
pub struct Page {
    data: [u8; 4096],
    ref_count: usize,
    ppn: u32,
}

impl Page {
    pub(crate) fn new(ppn: u32) -> Self {
        Self {
            data: [0; 4096],
            ref_count: 0,
            ppn: ppn
        }
    }

    pub fn read<T>(&self, index: usize) -> &[u8] {
        if self.ppn == 0 {
            return &[0];
        }

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

    pub fn ref_count(&self) -> usize {
        self.ref_count
    }

    pub fn increment_refs(&mut self) {
        self.ref_count += 1
    }

    pub fn decrement_refs(&mut self) {
        self.ref_count -= 1
    }
}

struct Memory {
    free_list: Vec<Page>,
    zero_page: &'static Page,
}

impl Memory {
    fn new() -> Self {
        let mut v: Vec<Page> = Vec::new();
        for ppn in 1..32 {
            v.push(Page::new(32 - ppn));
        }
        Self {
            free_list: v,
            zero_page: &ZERO_PAGE,
        }   
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop().map(|mut page| {
            page.zero();
            page
        })
    }

    fn push_free(&mut self, mut page: Page) {
        page.zero();
        self.free_list.push(page);
    }

    fn get_zero_ref(&self) -> &Page {
        self.zero_page
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
                Some(&mut page as *mut Page)
            },
            None => None
        }
    }
}

pub fn kfree(page: &mut Page) {
    if page.ppn == 0 {
        return;
    }
    unsafe {
        MEM.push_free(*page);
    }
}

pub fn zero_page() -> &'static Page {
    let zero: &Page;
    zero = unsafe { MEM.get_zero_ref() };
    zero
}