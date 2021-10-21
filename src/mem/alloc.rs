use std::vec::Vec;

static ZERO_PAGE: Page = Page {
    data: [0; 4096],
    ref_count: 0,
    ppn: 0
};

static mut MEM: Memory = Memory {
    free_list: Vec::new(),
    used_list: Vec::new(),
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

    pub fn ppn(&self) -> u32 { self.ppn }

    fn zero(&mut self) {
        self.write::<[u8; 4096]>(0, &[0; 4096]);
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

    pub fn copy(&mut self, other: &Page) {
        self.write::<[u8; 4096]>(0, &other.data);
    }
}

struct Memory {
    free_list: Vec<Page>,
    used_list: Vec<Page>,
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
            used_list: Vec::new(),
            zero_page: &ZERO_PAGE,
        }   
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop().map(|mut page| {
            page.zero();
            page
        })
    }

    fn push_free(&mut self, page: Page) {
        self.free_list.push(page);
    }

    fn push_used(&mut self, page: Page) {
        self.used_list.push(page);
    }

    fn remove_used(&mut self, page: &Page) -> Option<Page> {
        let mut idx = self.used_list.len();
        for i in 0..self.used_list.len() {
            if self.used_list[i].ppn == page.ppn {
                idx = i;
            }
        }
        if idx != self.used_list.len() {
            return Some(self.used_list.remove(idx));
        }
        return None;
    }

    fn used_peek(&mut self) -> Option<&mut Page> {
        let idx = self.used_list.len() - 1;
        self.used_list.get_mut(idx)
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

pub fn kalloc() -> Option<&'static mut Page> {
    unsafe {
        match MEM.pop_free() {
            Some(mut page) => {
                page.increment_refs();
                MEM.push_used(page);
                MEM.used_peek()
            },
            None => None
        }
    }
}

pub fn kfree(page: &Page) {
    if page.ppn == 0 {
        return;
    }
    unsafe {
        MEM.push_free(MEM.remove_used(page).unwrap());
    }
}

pub fn zero_page() -> &'static Page {
    let zero: &Page;
    zero = unsafe { MEM.get_zero_ref() };
    zero
}