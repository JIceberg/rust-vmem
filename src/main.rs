mod mem;
mod sim;
use sim::check::{ValueType, DataType, Simulator};
use sim::pointer::Pointer;
mod proc;

fn main() {
    let mut sim = Simulator::begin(true);

    let mut x = ValueType::Zero;
    let ptr_x = Pointer::new(&mut x);

    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::SignedInt(-2));
    match sim.read(ptr_x, DataType::SignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value() as isize),
        None => {}
    };
    
    let mut y = ValueType::Zero;
    let ptr_y = Pointer::new(&mut y);

    sim.write(ptr_y, ValueType::UnsignedInt(2));
    match sim.read(ptr_y, DataType::UnsignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value()),
        None => {}
    };

    sim.fork();
    sim.write(ptr_x, ValueType::UnsignedInt(3));
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };
    match sim.read(ptr_y, DataType::SignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value()),
        None => {}
    };


    println!();
    sim.print();
}
