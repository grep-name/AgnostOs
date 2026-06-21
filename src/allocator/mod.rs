use core::alloc::GlobalAlloc;
use core::alloc::Layout;

struct FreeChunk {
    size: usize,
    next: *mut FreeChunk,
}

pub struct AgnostosAllocator {
    head: spin::Mutex<*mut FreeChunk>,
}

// Since we are only one core they are here just so rust compiler would shut up and dont do anything.
unsafe impl Send for AgnostosAllocator {}
unsafe impl Sync for AgnostosAllocator {}

impl AgnostosAllocator {
    pub const fn new() -> Self {
        Self {
            head: spin::Mutex::new(core::ptr::null_mut()),
        }
    }

    pub fn init(&self, heap_start: usize, heap_size: usize) {
        unsafe {
            let chunk = heap_start as *mut FreeChunk;
            (*chunk).size = heap_size;
            (*chunk).next = core::ptr::null_mut();
            *self.head.lock() = chunk;
        }
    }
}

unsafe impl GlobalAlloc for AgnostosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = layout.size().max(core::mem::size_of::<FreeChunk>());
            let align = layout.align().max(core::mem::align_of::<FreeChunk>());

            let mut head = self.head.lock();
            let mut current: *mut *mut FreeChunk = &mut *head;
            let mut prev: *mut *mut FreeChunk = &mut *head;

            while !(*current).is_null() {
                let chunk = *current;
                let start = chunk as usize;
                let aligned = (start + align - 1) & !(align - 1);
                let end = aligned + size;
                let chunk_end = start + (*chunk).size;

                if end <= chunk_end {
                    // this chunk fits — check if remainder is big enough to keep
                    let remainder_start = end;
                    let remainder_size = chunk_end - end;

                    if remainder_size >= core::mem::size_of::<FreeChunk>() {
                        // split: put remainder back as a new free chunk
                        let remainder = remainder_start as *mut FreeChunk;
                        (*remainder).size = remainder_size;
                        (*remainder).next = (*chunk).next;
                        *prev = remainder;
                    } else {
                        // too small to split, remove chunk entirely
                        *prev = (*chunk).next;
                    }

                    return aligned as *mut u8;
                }
                prev = &mut (*chunk).next;
                current = &mut (*chunk).next;
            }
        }
        return core::ptr::null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            let size = layout.size().max(core::mem::size_of::<FreeChunk>());
            let mut head = self.head.lock();

            // write a FreeChunk header at the freed pointer
            let chunk = ptr as *mut FreeChunk;
            (*chunk).size = size;

            // insert at head of free list
            (*chunk).next = *head;
            *head = chunk;
        }
    }
}
