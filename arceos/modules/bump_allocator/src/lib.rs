#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    mem: [Block<PAGE_SIZE>; MAP_LEN],
    block_num: usize,
}

const MAP_LEN: usize = 1 << 10;

#[repr(C)]
pub struct Block<const PAGE_SIZE: usize> {
    start: usize,
    size: usize,
    b_used: usize,
    p_used: usize,
}

impl<const PAGE_SIZE: usize> Block<PAGE_SIZE> {
    pub const fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
            b_used: 0,
            p_used: 0,
        }
    }

    pub fn available_bytes(&self) -> usize {
        self.size - self.b_used - self.p_used * PAGE_SIZE
    }

    pub fn alloc_bytes(&mut self, size: usize) -> *mut u8 {
        let ptr = self.start + self.b_used;
        self.b_used += size;
        ptr as *mut u8
    }

    pub fn available_pages(&self) -> usize {
        self.available_bytes() / PAGE_SIZE
    }

    pub fn alloc_pages(&mut self, size: usize) -> usize {
        let end = self.start + self.size;
        let page_pos = end - size * PAGE_SIZE;
        let new_page_pos = page_pos - size * PAGE_SIZE;
        let ptr = new_page_pos;
        self.p_used += size;
        ptr
    }
}

impl<const P: usize> EarlyAllocator<P> {
    pub const fn new() -> Self {
        let arr: [Block<P>; MAP_LEN] = unsafe { core::mem::zeroed() };
        Self {
            mem: arr,
            block_num: 0,
        }
    }
}

impl<const P: usize> BaseAllocator for EarlyAllocator<P> {
    fn init(&mut self, start: usize, size: usize) {
        self.add_memory(start, size).unwrap();
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        self.mem[self.block_num] = Block::new(start, size);
        self.block_num += 1;
        Ok(())
    }
}

impl<const P: usize> ByteAllocator for EarlyAllocator<P> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let size = layout.size();
        for i in 0..self.block_num {
            if self.mem[i].available_bytes() >= size {
                return Ok(core::ptr::NonNull::new(self.mem[i].alloc_bytes(size))
                    .ok_or(allocator::AllocError::NoMemory)?);
            }
        }
        Err(allocator::AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        // nothing to do
    }

    fn total_bytes(&self) -> usize {
        self.mem.iter().map(|block| block.size).sum()
    }

    fn used_bytes(&self) -> usize {
        self.mem.iter().map(|block| block.b_used).sum()
    }

    fn available_bytes(&self) -> usize {
        self.mem.iter().map(|block| block.available_bytes()).sum()
    }
}

impl<const P: usize> PageAllocator for EarlyAllocator<P> {
    const PAGE_SIZE: usize = P;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        for i in 0..self.block_num {
            if self.mem[i].available_pages() >= num_pages {
                return Ok(self.mem[i].alloc_pages(num_pages));
            }
        }
        Err(allocator::AllocError::NoMemory)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // nothing to do
    }

    fn total_pages(&self) -> usize {
        self.mem.iter().map(|block| block.size).sum()
    }

    fn used_pages(&self) -> usize {
        self.mem.iter().map(|block| block.p_used).sum()
    }

    fn available_pages(&self) -> usize {
        self.mem.iter().map(|block| block.available_pages()).sum()
    }
}
