# Processes

The processes in this simulated virtual memory scheme handle all the main virtual memory
features and implementations for the simulation.

## States

A process can be sleeping, running, or terminated. When a process exists and has context in the userspace
but isn't running, the process is asleep. This means that it will not run any of its logic as
it is not valid to call to a sleeping process. When context is switched to that process,
it wakes up and starts running. When the process is killed, it terminates, gets set to terminated state,
and has its page directory, page tables, physical pages freed.

Physical pages, however, can have multiple references, so it's not a good idea to free them right away.
Instead, we decrement the reference count.

## Page Faults

Instead of forwarding to some trapframe to go from user mode to kernel on an invalid write
or page fault, page faults occur and are resolved at the process-level for this simulation. For example,
if our process attempts to write to a page whose PTE writable flag is not enabled, the page has been copied
by a forked process. But it's impossible with the current implementation to know which process is the
parent and which is the child. The process has no knowledge of this information.
No process "owns" pages. All pages are mapped via references to memory locations in static memory;
in reality, pages are not stored in static memory and instead are actual
memory locations on the device. Here, we are writing to simulated pages that model our computer memory.

Page faults happen solely on writes. No page fault happens on a read, because reading memory
does not change anything in the system, making it process-agnostic. Writing, however, does
change the system; so, whenever we write to memory, we need to ensure that the write is valid and will
continue to handle it until it actually writes to the memory.

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

## Context

Each process has a _context_ that models its current state. From a user perspective, the program
can switch between running processes seemlessly through the `switch(n)` simulation command.
Below is an example of quick swapping between processes at the user's whim to write data to a shared
value,

```rust
let mut sim = Simulator::begin(true);

let mut x = ValueType::Zero;
let ptr_x = Pointer::new(&mut x);

sim.register(ptr_x);
match sim.read(ptr_x, DataType::UnsignedInt) {
    Some(value) => println!("Value of x: {}", value.get_value()),
    None => {}
};

sim.fork();
sim.fork();
sim.fork();

sim.switch(0);

for i in 0..1024 {
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x before write for pid {}: {}", i%4, value.get_value()),
        None => {}
    };
    sim.write(ptr_x, ValueType::UnsignedInt(i));
    match sim.read(ptr_x, DataType::UnsignedInt) {
        Some(value) => println!("Value of x after write for pid {}: {}", i%4, value.get_value()),
        None => {}
    };
    sim.switch((i+1)%4);
}
```

The output will look something like this,
```
Value of x: 0
Value of x before write for pid 0: 0
PGZERO: 0x0
Value of x after write for pid 0: 0
Value of x before write for pid 1: 0
PGZERO: 0x0
Value of x after write for pid 1: 1
Value of x before write for pid 2: 0
PGZERO: 0x0
Value of x after write for pid 2: 2
Value of x before write for pid 3: 0
PGZERO: 0x0
Value of x after write for pid 3: 3
Value of x before write for pid 0: 0
Value of x after write for pid 0: 4
Value of x before write for pid 1: 1
Value of x after write for pid 1: 5
Value of x before write for pid 2: 2
Value of x after write for pid 2: 6
...
```

Each process generates a page fault (PGZERO) on the initial write. 
That's because they are all initially referencing the zero page for `x`, which is read-only.
You can learn more about how the zero page is handled exactly in the [paging](paging) reference.

It's important to note that each process is looking solely at its own state.
PID 0 and 2 are looking at _different_ values for `x` despite it being the same virtual address!
In a paging scheme, the virtual address gets translated into the page directory and
the page tables for the current process. Each process has its own page directory and page tables.
This is its context; the way the mapping between the virtual and physical address works becomes different.
Now each process is no longer working with the same location in local memory.

In this example, let's remark that PID 2 is using page 11 and that PID 0 is using page 9.
These physical pages have different physical page numbers (PPN), so the page table
that holds the entry referring to our virtual address for `x` is different.
The page table entry contains the physical page number being referenced and some flags
that aid in managing memory for things like page faults. When the virtual address is being
looked up with respect to the current process's context, when we reach the page table entry,
the final translation is different. The physical address is formed by taking the PPN
from the PTE as the upper 20 bits and the offset from the virtual address as the lower 12.
If the PPN is different, then it will refer to a different page number. A more detailed explanation
as to how all these mappings work can be found in the [paging](paging) reference.

If the user so wished it, they could create their own process scheduler just by using this basic
simulation.
