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
