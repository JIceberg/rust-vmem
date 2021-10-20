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
    let mut sim = Simulator::begin();

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
* Page swapping

All of these features are handled by the simulated processes to enable page faults.
This avoids needing a trapframe implementation.

### Copy-on-Write

A child process and its parent initially point to the same physical pages to avoid
costly copying of all the data in the parent's pages to owned pages for the
child. These pages are read-only. On a write from either the parent or the child to a page, the process that
performed the write will copy the page and perform a remap. The new page will be write enabled
but the old page will still be read-only. Grandchild and sibling process can also reference
the same physical page.

### Zero-Initialized Data

All data in physical memory is initialized to 0. All initially allocated pages prior to any
write refer to a universal zero page in memory which cannot be evicted or modified.

### Lazy Page Allocation

When a process attempts to write to a virtual address that is mapped to the zero page, it allocates
a new page and remaps the virtual address to the new physical page. This saves on costly
allocations to where we only allocate pages for a process when they begin using them.

### Page Swapping
There is a maximum amount of RAM that this simulation allocates for physical pages.
When we run out of physical memory for the pages, some pages will be evicted such that
their contents get written to a swap disk and then the page is freed. Once it is freed, it gets
put back into the free list for whatever process is running to make use of.

This needs to consider pages that have been written to. The dirty flag on a page table entry
tells the eviction program that the page needs to be written to the disk, as since it was last there
it was modified. In the case that the dirty bit is clear, the page replacement algorithm will
simply evict the page as its contents are already in the disk.

Pages with the protected flag enabled cannot be evicted. This includes page directories, page tables,
and the universal zero page.
