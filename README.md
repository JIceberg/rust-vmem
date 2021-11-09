# Virtual Memory in Rust

A virtual memory implementation in Rust, complete with paging and memory allocation techniques.

## Background

This virtual memory scheme uses two-level paging to manage 32-bit byte-addressable memory
for a basic simulated kernel. Processes are hard simulated through commands that emulate
basic process interaction with the kernel and memory. The following are commands available
to the user:

| Command | Description |
|---|---|
| `register(addr)` | Registers the given virtual address `addr` to the current process. |
| `write(addr, value)` | Writes `value` to the given virtual address `addr`. The address must be valid for the process. |
| `read(addr)` | Returns the value stored at the given virtual address `addr`. The address must be valid for the process. |
| `fork()` | Forks a child process from the current running process. Yields context to the child until it dies. |
| `switch(n)` | Switches the process to the n-th running address. |
| `kill()` | Kills the process. |

An example implementation the user might do for creating variables is below,
```rust
fn main() {
    let mut sim = Simulator::begin(false);

    let mut x = ValueType::UnsignedInt(0);
    let mut y = ValueType::UnsignedInt(1);
    let ptr_x = Pointer::new(&mut x);
    let ptr_y = Pointer::new(&mut y);

    // register and write to the variable
    sim.register(ptr_x);
    sim.write(ptr_x, ValueType::UnsignedInt(5));

    // this should print 5
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x: {}", value.get_value()),
        None => {}
    };

    // This is valid, because we registered
    // ptr_x which creates a page of data.
    // When that page is allocated, all virtual addresses
    // within the page boundary will properly map to said page.
    sim.register(ptr_y);    // will say "already registered"
    sim.write(ptr_y, ValueType::UnsignedInt(6));
    match sim.read(ptr_y, DataType::UnsignedInt) {
        Some(value) => println!("Value of y: {}", value.get_value()),
        None => {}
    };

    for _ in 0..1024 {
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
```

For every variable you create, you need to register a pointer to that variable before
working with that variable. When you register the address, the process allocates
a page (if there is not already a page available) for that virtual address and produces a mapping.
Once registered, you can then write to the address or read from it. Attempting to write to an invalid
address does not panic but prints a warning stating that the address is invalid.
Reading an address in the simulator will return an `Option` that will either contain
the value stored at the address or nothing in the case that the address is not registered.

There are more advanced programs and simulations the user can create using the simulator
which are expanded upon further in other readings.

## Features

This simulated virtual memory has the following features:

* Copy-on-write
* Zero-initialized data
* Lazy page allocation

Potential future features:

* Caching
* Page replacement

All of these features are handled by the simulated processes to enable page faults.
This avoids needing a trapframe implementation. You can read more about these
features in the [docs](docs/processes.md).