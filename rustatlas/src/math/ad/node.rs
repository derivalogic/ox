#[derive(Default, Clone)]
pub struct TapeNode {
    pub childs: Vec<usize>, // indices of children on the tape
    pub derivs: Vec<f64>,   // matching ∂parent / ∂child
    pub adj: f64,           // this node’s adjoint
}

impl TapeNode {
    #[inline]
    pub fn propagate_into(&self, tape: &mut [TapeNode]) {
        let a = self.adj;
        for (&c, &d) in self.childs.iter().zip(&self.derivs) {
            tape[c].adj += a * d;
        }
    }
}
