use super::ad::num::Real;

#[inline]
fn norm_pdf<T: Real>(x: T) -> T {
    Real::exp(-(x * x) * 0.5) / T::from((2.0 * std::f64::consts::PI).sqrt())
}

#[inline]
fn norm_cdf<T: Real>(x: T) -> T {
    let k = T::from(1.0) / (T::from(1.0) + T::from(0.2316419) * x.abs());
    let k_sum = k
        * (T::from(0.31938153)
            + k * (T::from(-0.356563782)
                + k * (T::from(1.781477937)
                    + k * (T::from(-1.821255978) + k * T::from(1.330274429)))));
    let approx = T::from(1.0) - norm_pdf(x) * k_sum;
    if x >= T::from(0.0) {
        approx
    } else {
        T::from(1.0) - approx
    }
}

/// Black-Scholes call price and greeks (delta, gamma, theta)
pub fn call_price_greeks<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> (T, T, T, T) {
    let sqt = t.sqrt();
    let d1 = ((s / k).ln() + (r + T::from(0.5) * vol * vol) * t) / (vol * sqt);
    let d2 = d1 - vol * sqt;
    let price = s * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2);
    let delta = norm_cdf(d1);
    let gamma = norm_pdf(d1) / (s * vol * sqt);
    let theta =
        -s * norm_pdf(d1) * vol / (T::from(2.0) * sqt) - r * k * (-r * t).exp() * norm_cdf(d2);
    (price, delta, gamma, theta)
}
