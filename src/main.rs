mod mem;
use mem::translation::Address;
use mem::ptable::PageTable;

extern crate rand;
use rand::Rng;

fn main() {
    let mut directory: PageTable = [0; 1024];
    let mut tables = [[0; 1024]; 1024];
    let mut rng = rand::thread_rng();
    for i in 0..1024 {
        directory[i] = (i << 12) as u32;
        for j in 0..1024 {
            tables[i][j] = rng.gen::<u32>();
        }
    }

    let v = Address::Virtual(rng.gen::<u32>());
    println!("Virtual address: 0x{:x}", v.get_address());
    println!("Physical address: 0x{:x}", v.translate(&directory, &tables).get_address());
}
