mod mem;
mod sim;
use sim::check::{ValueType, DataType, Simulator};
use sim::pointer::Pointer;
mod proc;

fn main() {
    let mut sim = Simulator::begin();

    let mut x = ValueType::Zero;
    let ptr_x = Pointer::new(&mut x);

    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::SignedInt(-2));
    match sim.read(ptr_x, DataType::SignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value() as isize),
        None => {}
    };

    

    println!();
    sim.print();
}
