mod mem;
mod sim;
use sim::check::{ValueType, DataType, Simulator};
use sim::pointer::Pointer;
mod proc;

fn main() {
    let mut sim = Simulator::begin();

    let mut x = ValueType::UnsignedInt(0);
    let ptr_x = Pointer::new(&mut x);

    // register and write to the variable
    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::UnsignedInt(5));
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    sim.fork();
    
    sim.switch(0);
    sim.write(ptr_x, ValueType::UnsignedInt(6));
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    for _ in 0..256 {
        let mut z = ValueType::UnsignedInt(0);
        let _ = Pointer::new(&mut z);   // iterate to the next page boundary
    }

    sim.switch(1);
    sim.write(ptr_x, ValueType::UnsignedInt(4));
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    sim.kill();
    
    let mut y = ValueType::UnsignedInt(0);
    let ptr_y = Pointer::new(&mut y);
    sim.register(ptr_y);

    sim.write(ptr_y, ValueType::SignedInt(-1));
    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value() as isize),
        None => {}
    };

    sim.fork();
    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value() as isize),
        None => {}
    };

    sim.write(ptr_y, ValueType::SignedInt(-2));
    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value() as isize),
        None => {}
    };

    sim.kill();
    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value() as isize),
        None => {}
    };
    sim.fork();

    for _ in 0..256 {
        let mut z = ValueType::UnsignedInt(0);
        let _ = Pointer::new(&mut z);   // iterate to the next page boundary
    }

    let mut z = ValueType::UnsignedInt(0);
    let ptr_z = Pointer::new(&mut z);

    sim.register(ptr_z);
    sim.write(ptr_z, ValueType::UnsignedInt(3));
    match sim.read(ptr_z, DataType::UnsignedInt) {
        Some(value) => println!("Value of z: {}", value.get_value()),
        None => {}
    };

    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value() as isize),
        None => {}
    };

    println!();
    sim.print();
}
