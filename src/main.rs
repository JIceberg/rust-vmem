mod mem;
mod sim;
use sim::check::{ValueType, Simulator};
use sim::pointer::Pointer;
mod proc;

fn main() {
    let mut sim = Simulator::begin();

    let mut x = ValueType::UnsignedInt(0);
    let mut y = ValueType::UnsignedInt(1);
    let ptr_x = Pointer::new(&mut x);
    let ptr_y = Pointer::new(&mut y);

    // register and write to the variable
    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::UnsignedInt(5));

    // this should print 5
    match sim.read(ptr_x) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    // This is valid, because we registered
    // ptr_x which creates a page of data
    // when that page is allocated, all virtual addresses
    // will properly map to said page.
    sim.register(ptr_y);    // will say "already registered"
    sim.write(ptr_y, ValueType::UnsignedInt(6));
    match sim.read(ptr_y) {
        Some(value) => println!("Value of y: {}", value.get_value()),
        None => {}
    };

    for _ in 0..4096 {
        let mut z = ValueType::UnsignedInt(2);
        let _ = Pointer::new(&mut z);   // this is just to increment our simulated virtual address
    }
    let mut z = ValueType::SignedInt(2);
    let ptr_z = Pointer::new(&mut z);

    // This is invalid, because we have crossed the boundary
    // of the last allocated page, so we need to register this pointer
    // so we can allocate a new page
    sim.write(ptr_z, ValueType::SignedInt(-1));
}
