use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

const HEAP_SIZE: usize = 1024 * 1024; // 1MB

struct BumpAllocator {
    heap: UnsafeCell<[u8; HEAP_SIZE]>,
    offset: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let offset = *self.offset.get();
            let align = layout.align();
            let aligned = (offset + align - 1) & !(align - 1);
            let new_offset = aligned + layout.size();
            if new_offset > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            *self.offset.get() = new_offset;
            (self.heap.get() as *mut u8).add(aligned)
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocator, no dealloc
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap: UnsafeCell::new([0u8; HEAP_SIZE]),
    offset: UnsafeCell::new(0),
};