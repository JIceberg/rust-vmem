# Paging

The paging scheme for this 32-bit simulated virtual memory system is a two-level paging mechanism
where processes have their own directory and tables but they all share (for the most part) the same
physical pages until a write occurs.

## Translation

The translation between physical and virtual addresses occur in a page walk, where we take the address
and look through the page directory and page tables as needed. To do so, we have an indexing method where
each page table (including the directory) is represented as an array of 4096 bytes (`u8`), since our memory
is byte-addressable. But this is not useful for the purpose of retrieving information on pages.
It's just memory. So we need to read that memory and represent it as something in a way that is useful.
Each entry in the table is 32 bits, with the upper 20 bits being an indexing number and the lower 12 being
metadata flags. In memory, these are stored as 4 bytes in little endian and then represented as a
`PageTableEntry` type when read to retrieve useful output.

### Page Directory Entry (PDE)

A program to retrieve a page directory entry could be,

```rust
// va is our virtual address
let pdx = va.get_dir_index();

// d is the page directory for our process
let raw_pd_data = d.read::<u32>(pdx * 4);
let pde = PTE::from(raw_to_u32(raw_pd_data));
```

where `PTE` is an alias type for `PageTableEntry`. We can use this to represent our
page directory entry because the page directory is just the first-level page table
given a unique name to differentiate itself from the second-level page tables.
In this program, `d` is our page directory for the current process represented as a
`Page` type which is just an abstraction for our array of 4096 bytes. It's important
to note that our `PTE` type is also just an abstraction for a 32-bit unsigned integer
(`u32`) that makes it useful for the purposes of our virtual memory mechanism.

The `read<T>(idx)` function for a `Page` looks at the size-aligned
index for some type T and returns however many bytes are needed to represent
that object in little endian. The code for this is relatively simple,

```rust
fn read<T>(&self, index: usize) -> &[u8] {
    let size = std::mem::size_of::<T>();
    let s = index - index % size;
    let e = s + size;

    &self.data[s..e]
}
```

where `data` is our array of 4096 bytes. We can then read through these raw bytes as such,

```rust
fn raw_to_u32(raw_data: &[u8]) -> u32 {
    let mut shif = 0;
    let mut val: u32 = 0;
    for i in 0..raw_data.len() {
        val |= (raw_data[i] as u32) << shif;
        shif += 8;
    }
    val
}
```

and now we have a 32-bit unsigned integer we can use to create our `PTE`. Note the index we used
to retrieve this directory entry: `va.get_dir_index()`. `va` is an `Address` type, which is yet another
abstraction for a 32-bit unsigned integer. This returns the upper 10 bits of the virtual address
and is used to represent which index entry we are using. So, the virtual address 0x80000000
would direct to the 512th page directory entry. We multiply this by 4 because our memory is byte-addressable,
so each entry is stored in memory in 4 byte alignment. There are 10 bits we use for the offset,
which gives us a range of values between 0 and 1023 for an unsigned integer. Multiplying by 4
gives us a range of values between 0 and 4095, and then we look at the closest word-aligned (a word is 4 bytes)
index into memory that is less than the index; hence, `s = index - index % size` which performs
this index alignment.

### Page Table Entry (PTE)

The way of getting a page table entry is the same as retrieving the page directory entry.
Each page directory has a PTN (page table number) which is used to index into
the process's page tables. We then index into our page table using bits 12 to 21
of the virtual address through `va.get_table_index()`. A program to retrieve our `PTE`
from this page table is,

```rust
// va is our virtual address
let ptx = va.get_table_index();

// self.tables is where the process stores
// its references to the pages it has allocated for
// its page tables
let pgtab = self.tables[pde.get_ppn()].borrow();
        
let raw_data = pgtab.read::<u32>(ptx * 4);
let pte = PTE::from(raw_to_u32(raw_data));
```

### Physical Address (PA)

```rust
let pa = Physical::from(
    pte.get_ppn(),      // the page number for our page
    va.get_offset(),    // the offset into our physical page (index)
    ..                  // reserved for the sake of the simulation
).get();
```

If this weren't a simulation and instead an actual implementation of virtual memory,
we would simply just treat this physical address as a pointer and write/read with that.
But because this is a simulation that runs solely at user-level, we cannot do that,
but the point and idea is clear regardless.


## Using a `PageTableEntry`

The `PageTableEntry` type (aliased by `PTE`) is an abstraction for a 32-bit unsigned
integer for the purpose of making our virtual mechanism readable, concise, and higher level.
The upper 20 bits of a `PTE` is the PPN (physical page number) and the lower 12 bits
are metadata flags for memory management. These flags have also been abstracted into an
enum type `Flag`.

| Flag | Description |
| --- | --- |
| Present | If set, the entry is referencing an existing page. |
| Writable | If set, the page being referenced can receive writes. |
| User | If set, this page is allocated for a user process. |
| WriteThrough | If set, this page is cached via write-through to the disk. Otherwise, the caching is write-back. |
| CacheDisable | If set, this page is not to undergo caching to the disk. |
| Accessed | If set, the page is accessed by a non-terminated process. |
| Dirty | If set, this page has been written to by a process. |
| Protected | If set, this page cannot be evicted by page replacement. |
| Zero | If set, this entry is referencing the 0 page. |


## Page Faults

Page faults happen solely on writes. No page fault happens on a read, because reading memory
does not change anything in the system, making it process-agnostic. Writing, however, does
change the system; so, whenever we write to memory, we need to ensure that the write is valid and will
continue to handle it until it actually writes to the memory.

### Copy-on-Write

When we fork from a process, a new child process is constructed. We allocate
pages for the new process's page directory and page tables and use them to make
copies of the parent's. This is all done in the `copy` function.

On a write to the process, we perform our typical checks. In the case that the page
is not the zero page yet is not writable, it means our page has either been copied or is copied.
It is not possible to know which, but we can know if it is currently being referenced by many or
one processes. Our memory has metadata we associate with each `Page` type. Recall that a `Page`
is a wrapper for a 4096-byte chunk of memory, but we also include metadata for the page in that
struct type. This metadata keeps track of how many references the page has, which is useful
for our copy-on-write implementation.
In the case where there is only one process referencing the page, the handle is simple.
What we do is mark the page as writable and then attempt the write again. This reattempt
is a simulated version of the page fault yielding back to the process which then reattempts the write.

The case where there _are_ multiple processes referencing this page is more complicated.
What we're doing here is the actual copy on write--we've written to the page and so we need to change our mapping
from the shared page to a newly allocated physical page. We copy the contents of the old
page into this new page and then modify the corresponding page table entry so the mapping
refers to the new page instead of the old shared referenced one. We then decrease the number
of references to the old page by 1 because we are no longer referring to it in this
process and reattempt the write.

### Zero-Initialized Data

All data in physical memory is initialized to 0. All initially allocated pages prior to any
write refer to a universal zero page in memory which cannot be evicted or modified. On any attempt
to write to the zero page, a new page is lazily allocated, which is discussed in the next section.

### Lazy Page Allocation

When a process attempts to write to a virtual address that is mapped to the zero page, it allocates
a new page and remaps the virtual address to the new physical page. This saves on costly
allocations to where we only allocate pages for a process when they begin using them.
