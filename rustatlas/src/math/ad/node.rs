use std::{fmt, ptr::NonNull};

/// A node in the reverse-mode tape.
///
/// * `childs` now stores **raw pointers** to the child nodes instead of
///   integer indices.  The addresses are stable because every node lives in
///   a `Box` (or the bump-arena) for the whole lifetime of the tape.
#[derive(Clone)]
pub struct TapeNode {
    pub childs: Vec<NonNull<TapeNode>>, // ← was Vec<usize>
    pub derivs: Vec<f64>,               // ∂parent / ∂child
    pub adj: f64,
}

impl fmt::Debug for TapeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TapeNode {{ addr: {:?}, childs: {:?}, derivs: {:?}, adj: {} }}",
            self as *const Self as *const (), self.childs, self.derivs, self.adj
        )
    }
}

impl Default for TapeNode {
    fn default() -> Self {
        Self {
            childs: Vec::new(),
            derivs: Vec::new(),
            adj: 0.0,
        }
    }
}

impl TapeNode {
    /// Propagate this node’s adjoint into its children
    #[inline(always)]
    pub fn propagate_into(&self) {
        debug_assert_eq!(self.childs.len(), self.derivs.len());
        let a = self.adj;
        for (&child, &d) in self.childs.iter().zip(&self.derivs) {
            // Safe because every child pointer was produced by the same Tape
            // and remains valid for its whole lifetime.
            unsafe { (*child.as_ptr()).adj += a * d };
        }
    }
}
