use rustatlas::prelude::{NumericType, ToNumeric, max, min};

/// Functional if: smooth replacement for `if x > 0`.
/// Returns `b + (a - b)/eps * max(0, min(eps, x + eps/2))`.
///
/// # Arguments
/// - `x`: expression value
/// - `a`: value if `x` is positive
/// - `b`: value if `x` is negative
/// - `eps`: smoothing width
pub fn f_if(x: NumericType, a: NumericType, b: NumericType, eps: NumericType) -> NumericType {
    let half = eps / NumericType::new(2.0);
    let t = min(max(x + half, NumericType::zero()), eps);
    (b + (a - b) * t / eps).into()
}
