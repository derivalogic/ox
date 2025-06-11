//! tape.rs   – one rewindable reverse–mode tape **per thread**

use bumpalo::Bump;
use std::{cell::RefCell, ptr::NonNull};

use crate::prelude::TapeNode; // ← your node definition
use crate::utils::errors::{AtlasError, Result}; // ← your Result type
                                                /*───────────────────────────────────────────────────────────────────────────*/
/*  The tape itself                                                         */
/*───────────────────────────────────────────────────────────────────────────*/

pub struct Tape {
    bump: Bump,                   // arena for the nodes’ storage
    book: Vec<NonNull<TapeNode>>, // stable addresses of *all* nodes
    mark: usize,                  // “bookmark” for nested sweeps
    pub active: bool,             // are we currently recording?
}

/*── low-level helpers ─────────────────────────────────────────────────────*/

impl Tape {
    /// Allocate `n` in the bump-arena, remember its pointer, return it.
    #[inline(always)]
    fn push(&mut self, n: TapeNode) -> NonNull<TapeNode> {
        let ptr = NonNull::from(self.bump.alloc(n));
        self.book.push(ptr);
        ptr
    }

    #[inline(always)]
    pub fn reset_adjoints() {
        TAPE.with(|tc| {
            // we only need an immutable borrow to iterate over the pointers
            for &ptr in &tc.borrow().book {
                unsafe { (*ptr.as_ptr()).adj = 0.0 };
            }
        });
    }

    pub fn debug_print() {
        TAPE.with(|tc| {
            let tape = tc.borrow();
            for (i, &ptr) in tape.book.iter().enumerate() {
                let node = unsafe { ptr.as_ref() };
                println!("{}: {:?}", i, node);
            }
        });
    }

    #[inline(always)]
    fn index_of(&self, p: NonNull<TapeNode>) -> Option<usize> {
        self.book.iter().position(|&q| q == p)
    }
}

/*── public API (called from ADNumber / operators / sweeps) ────────────────*/

impl Tape {
    pub fn new() -> Self {
        Tape {
            bump: Bump::new(),
            book: Vec::new(),
            mark: 0,
            active: false,
        }
    }

    /// Make an *independent* leaf (no parents, zero adjoint).
    #[inline]
    pub fn new_leaf(&mut self) -> NonNull<TapeNode> {
        self.push(TapeNode::default())
    }

    /// Record a composite node while `self.active == true`.
    #[inline]
    pub fn record(&mut self, n: TapeNode) -> Option<NonNull<TapeNode>> {
        self.active.then(|| self.push(n))
    }

    /*----- optional node lookup helpers ----------------------------------*/
    pub fn node(&self, p: NonNull<TapeNode>) -> Option<&TapeNode> {
        self.index_of(p).map(|i| unsafe { self.book[i].as_ref() })
    }
    pub fn mut_node(&mut self, p: NonNull<TapeNode>) -> Option<&mut TapeNode> {
        self.index_of(p).map(|i| unsafe { self.book[i].as_mut() })
    }

    /*──────────────── reverse sweep ──────────────────────────────────────*/

    /// Back-propagate from `root` all the way to the beginning.
    pub fn propagate_from(&mut self, root: NonNull<TapeNode>) -> Result<()> {
        let start = self
            .index_of(root)
            .ok_or(AtlasError::NodeNotIndexedInTapeErr)?;
        for i in (0..=start).rev() {
            /*  Copy the node’s *value* so we only hold &TapeNode, not &mut,
            avoiding aliasing with the &mut self we already have.        */
            let node = unsafe { self.book[i].as_ref().clone() };
            node.propagate_into(); // ← NEW signature
        }
        Ok(())
    }

    /// Sweep from the last mark (exclusive) back to the start.
    pub fn propagate_mark_to_start(&mut self) {
        let end = self.mark.saturating_sub(1);
        for i in (0..=end).rev() {
            let node = unsafe { self.book[i].as_ref().clone() };
            node.propagate_into();
        }
    }

    pub fn start_recording() {
        TAPE.with(|tc| {
            let mut t = tc.borrow_mut();
            t.bump.reset(); // O(1) – frees nothing, just rewinds
            t.book.clear(); // forget every previous node pointer
            t.mark = 0;
            t.active = true;
        });
    }

    /// Leave recording mode but keep every node alive for back-prop.
    pub fn stop_recording() {
        TAPE.with(|tc| tc.borrow_mut().active = false);
    }

    /// Return `true` iff the current thread’s tape is recording.
    #[inline]
    pub fn is_active() -> bool {
        TAPE.with(|tc| tc.borrow().active)
    }

    /*— optional helpers the old API exposed —*/

    /// Set an absolute “bookmark” so later you can sweep only part of the tape.
    pub fn set_mark() {
        TAPE.with(|tc| {
            let len = tc.borrow().book.len();
            tc.borrow_mut().mark = len;
        });
    }

    /// Rewind the index book to the last mark (arena memory is kept).
    pub fn rewind_to_mark() {
        TAPE.with(|tc| {
            let mark = tc.borrow().mark;
            tc.borrow_mut().book.truncate(mark);
        });
    }

    /// Rewind everything back to an empty tape (fresh graph).
    pub fn rewind_to_init() {
        TAPE.with(|tc| {
            let mut t = tc.borrow_mut();
            t.bump.reset();
            t.book.clear();
            t.mark = 0;
        });
    }
}

/*───────────────────────────────────────────────────────────────────────────*/
/*  Thread-local singleton – one Tape per OS thread                         */
/*───────────────────────────────────────────────────────────────────────────*/

thread_local! {
    /// Each thread owns its *own* tape; no cross-thread aliasing.
    pub static TAPE: RefCell<Tape> = RefCell::new(Tape {
        bump:   Bump::new(),
        book:   Vec::new(),
        mark:   0,
        active: false,
    });
}
