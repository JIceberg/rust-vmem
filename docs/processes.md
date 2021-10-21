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
