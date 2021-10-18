extern crate raw_pointer as rptr;

use crate::mem::ptable::KERNBASE;
use core::ops::{Deref, DerefMut};
use core::convert::From;

pub struct Pointer<T> {
    vaddr: u32,
    ptr: rptr::Pointer<T>
}

static mut VADDR: u32 = KERNBASE;

impl<T> Pointer<T> {
    pub fn new(ptr: &mut T) -> Self {
        let vaddr = unsafe {
            VADDR += 4;
            VADDR
        };
        Self {
            vaddr,
            ptr: rptr::Pointer::new(ptr)
        }
    }

    pub fn vaddr(&self) -> u32 {
        self.vaddr
    }

    pub fn unwrap_mut(&self) -> &mut T {
        self.ptr.unwrap_mut()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Pointer<T> {
        Pointer::new(self.unwrap_mut())
    }
}

impl<T> Copy for Pointer<T> {}

impl<T> From<usize> for Pointer<T> {
    fn from(item: usize) -> Self {
        Self {
            vaddr: unsafe {
                VADDR += 4;
                VADDR
            },
            ptr: rptr::Pointer::from(item)
        }
    }
}

impl<T> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
       & *self.ptr
    }
}

impl<T> DerefMut for Pointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ptr
    }
}