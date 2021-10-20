mod mem;
mod sim;
use sim::check::{ValueType, DataType, Simulator};
use sim::pointer::Pointer;
mod proc;

fn main() {
    let mut sim = Simulator::begin();

    sim.fork();

    let mut x = ValueType::UnsignedInt(0);
    let ptr_x = Pointer::new(&mut x);

    // register and write to the variable
    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::UnsignedInt(5));

    // this should print 5
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    sim.kill();

    // register and write to the variable
    sim.register(ptr_x);

    // this should print 5
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    println!();
    sim.print();
}
