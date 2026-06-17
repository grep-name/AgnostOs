use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

pub struct SomethingAllocator {
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
}

impl SomethingAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: AtomicUsize::new(0),
            heap_end: AtomicUsize::new(0),
        }
    }

    pub fn init(&self, heap_start: usize, heap_size: usize) {
        self.heap_start.store(heap_start, Ordering::Relaxed);
        self.heap_end
            .store(heap_start + heap_size, Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for SomethingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let current = self.heap_start.load(Ordering::Relaxed);

        // Align the pointer
        let align = layout.align();
        let aligned = (current + align - 1) & !(align - 1);
        let next = aligned + layout.size();

        let end = self.heap_end.load(Ordering::Relaxed);

        if next > end {
            // Out of memory
            return core::ptr::null_mut();
        }

        self.heap_start.store(next, Ordering::Relaxed);

        return aligned as *mut u8;
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        todo!()
    }
}
