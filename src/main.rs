mod mem;
use mem::translation::Address;
use mem::ptable::{PageTable, PageTableEntry, Flag};

extern crate rand;
use rand::Rng;

fn main() {
    let mut directory: PageTable = [PageTableEntry::from(0); 1024];
    let mut tables = [[PageTableEntry::from(0); 1024]; 1024];
    let mut rng = rand::thread_rng();
    for i in 0..1024 {
        directory[i] = PageTableEntry::new(i as u32);
        for j in 0..1024 {
            tables[i][j] = PageTableEntry::from(rng.gen::<u32>());
        }
    }

    let v = Address::Virtual(rng.gen::<u32>());
    println!("Virtual address: 0x{:x}", v.get_address());
    println!("Physical address: 0x{:x}", v.translate(&directory, &tables).get_address());
}
