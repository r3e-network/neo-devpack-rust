//! Arena allocator for temporary translation objects (Round 83)
//!
//! This module provides a fast bump-pointer allocator for short-lived objects
//! during WASM translation, significantly reducing malloc/free overhead.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

const DEFAULT_BLOCK_SIZE: usize = 64 * 1024;
const ARENA_ALIGN: usize = 8;

/// Arena allocator for temporary objects
///
/// Round 83: Bump-pointer arena for fast temporary allocations
/// Round 84: Optimized layout for cache locality
pub struct Arena {
    // Use Cell/RefCell for interior mutability - allocation doesn't require &mut self
    current: Cell<Option<NonNull<u8>>>,
    pos: Cell<usize>,
    remaining: Cell<usize>,
    total_allocated: Cell<usize>,
    blocks: RefCell<Vec<(NonNull<u8>, Layout)>>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            current: Cell::new(None),
            pos: Cell::new(0),
            remaining: Cell::new(0),
            total_allocated: Cell::new(0),
            blocks: RefCell::new(Vec::with_capacity(4)),
        }
    }

    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        let size = layout.size();
        let align = layout.align();
        let old_pos = self.pos.get();
        let aligned_pos = (old_pos + align - 1) & !(align - 1);

        if aligned_pos + size > self.pos.get() + self.remaining.get() {
            self.grow(size.max(DEFAULT_BLOCK_SIZE));
        }

        let ptr = unsafe { self.current.get().unwrap().as_ptr().add(aligned_pos) };

        self.pos.set(aligned_pos + size);
        let consumed = (aligned_pos - old_pos) + size;
        self.remaining
            .set(self.remaining.get().saturating_sub(consumed));
        self.total_allocated.set(self.total_allocated.get() + size);

        unsafe {
            let ptr = ptr as *mut T;
            ptr.write(value);
            &mut *ptr
        }
    }

    #[cold]
    fn grow(&self, min_size: usize) {
        let size = min_size.next_power_of_two().max(DEFAULT_BLOCK_SIZE);
        let layout = Layout::from_size_align(size, ARENA_ALIGN).unwrap();

        unsafe {
            let ptr = NonNull::new(alloc(layout))
                .unwrap_or_else(|| std::alloc::handle_alloc_error(layout));
            self.current.set(Some(ptr));
            self.pos.set(0);
            self.remaining.set(size);
            self.blocks.borrow_mut().push((ptr, layout));
        }
    }

    pub fn reset(&self) {
        for (ptr, layout) in self.blocks.borrow_mut().drain(..) {
            unsafe {
                dealloc(ptr.as_ptr(), layout);
            }
        }
        self.current.set(None);
        self.pos.set(0);
        self.remaining.set(0);
        self.total_allocated.set(0);
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated.get()
    }

    pub fn block_count(&self) -> usize {
        self.blocks.borrow().len()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, DEFAULT_BLOCK_SIZE};

    #[test]
    fn alloc_consumes_remaining_capacity() {
        let arena = Arena::new();

        let _ = arena.alloc(1u8);
        assert_eq!(arena.remaining.get(), DEFAULT_BLOCK_SIZE - 1);

        let _ = arena.alloc(2u8);
        assert_eq!(arena.remaining.get(), DEFAULT_BLOCK_SIZE - 2);
    }
}
