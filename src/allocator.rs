use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

// 64KB heap in EWRAM
const HEAP_SIZE: usize = 64 * 1024;

// Wrapper for Sync
struct SyncHeap(UnsafeCell<[u8; HEAP_SIZE]>);
unsafe impl Sync for SyncHeap {}

#[unsafe(link_section = ".ewram")]
static HEAP_MEMORY: SyncHeap = SyncHeap(UnsafeCell::new([0; HEAP_SIZE]));

struct BumpAllocator {
    next: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let start = HEAP_MEMORY.0.get() as *mut u8;

            // Align the allocation
            let next = (*self.next.get() + layout.align() - 1) & !(layout.align() - 1);
            let end = next + layout.size();

            if end > HEAP_SIZE {
                // Out of memory
                return ptr::null_mut();
            }

            *self.next.get() = end;
            start.add(next)
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't deallocate
        // You could implement a more sophisticated allocator if needed
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator {
    next: UnsafeCell::new(0),
};

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    panic!("Allocation error");
}
