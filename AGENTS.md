# Hull-White + GBM Monte-Carlo Agent  

_Implementation notes & mathematical derivation_

---

## 1  Purpose  

This “agent” bundles everything you need to **simulate joint paths** of  

* the domestic **short-rate** \(r_t\) following a one–factor **Hull-White** model, and  
* an asset **price** \(S_t\) following a correlated **geometric Brownian motion (GBM)**,  

**under the risk-neutral measure** \(\mathbb{Q}\).  
Use the paths to price path-dependent derivatives, compute XVAs, Greeks, VaR, etc.

---

## 2  Stochastic models under \(\mathbb{Q}\)

### 2.1  Hull-White short rate  

\[
dr_t = \bigl(\theta(t) - a\,r_t\bigr)\,dt \;+\; \sigma_r \, dW_t^{(r)},
\]

* \(a>0\) mean-reversion speed  
* \(\sigma_r>0\) volatility  
* \(W^{(r)}\) is a Brownian motion under \(\mathbb{Q}\)  

The **bank-account numéraire** is  

\[
B_t \;=\; \exp\!\Bigl(\int_0^t r_s\,ds\Bigr),\qquad dB_t = r_t\,B_t\,dt,\; B_0=1.
\]

### 2.2  Asset price (equity, FX, commodity, …)

\[
dS_t = S_t\,r_t\,dt \;+\; S_t\,\sigma_S\,dW_t^{(S)},\qquad S_0>0,
\]

so the drift equals \(r_t\) (no-arbitrage because we discount with \(B_t\)).  
Brownian motions are correlated:

\[
dW_t^{(S)}\,dW_t^{(r)} = \rho\,dt,\qquad \rho\in[-1,1].
\]

---

## 3  Exact one-step discretisation  

Let \(\Delta t\) be a time step.

### 3.1  Rate transition  

The affine Hull-White model has a **closed-form step**:

\[
r_{t+\Delta t} =
r_t\,e^{-a\Delta t}
+\theta^\*\!(t,\Delta t)\,(1-e^{-a\Delta t})
+\sigma_r\,\sqrt{\frac{1-e^{-2a\Delta t}}{2a}}\;Z_1,
\]

\(Z_1\sim\mathcal N(0,1)\) and  

\[
\theta^\*(t,\Delta t)=\theta(t+\Delta t)
-\frac{\sigma_r^{\,2}}{2a^2}(1-e^{-a\Delta t})^{2}.
\]

### 3.2  Integrated rate (needed for discounting)

\[
\int_t^{t+\Delta t}\!r_s\,ds
=\frac{1-e^{-a\Delta t}}{a}\,r_t
+\Bigl(\Delta t - \frac{1-e^{-a\Delta t}}{a}\Bigr)\theta^\*(t,\Delta t)
+\sigma_r\,\sqrt{\frac{1-e^{-2a\Delta t}}{a^{2}}}\;Z_1,
\]

(where we reuse the same \(Z_1\) for efficiency and exact covariance).

### 3.3  Asset price update  

\[
\ln S_{t+\Delta t} =
\ln S_t
+\bigl(r_t - \tfrac12\sigma_S^{2}\bigr)\Delta t
+\sigma_S\Bigl(\rho\,Z_1+\sqrt{1-\rho^{2}}\,Z_3\Bigr)\sqrt{\Delta t},
\]

with an independent \(Z_3\sim\mathcal N(0,1)\).

---

## 4  Monte-Carlo algorithm (per path)

1. **Initialise** \((r_0,S_0,B_0=1)\).  
2. For each step \(k\):  
   * draw \(Z_1, Z_3\);  
   * update \(r\) (3.1);  
   * accumulate discount factor via the integral (3.2);  
   * update \(S\) (3.3).  
3. Evaluate payoff \(f(S_{\bullet}, r_{\bullet})\) and discount with product of factors.  
4. **Present value** = sample mean of discounted payoffs.  

Cost: \(\mathcal O(N\,n)\) for \(N\) paths, \(n\) steps.

---

## 5  Python skeleton  

```python
import numpy as np

class HullWhiteGBMAgent:
    """
    Simulates correlated Hull-White short rate and GBM asset
    under the risk-neutral measure using exact rate steps.
    """
    def __init__(self, a, sigma_r, theta,
                 sigma_s, rho,
                 t_grid, r0, s0,
                 rng=np.random.default_rng()):
        """
        theta : callable giving theta(t)
        t_grid: 1-D array of times [t0,…,tN] in years
        """
        self.a, self.sigma_r, self.theta = a, sigma_r, theta
        self.sigma_s, self.rho = sigma_s, rho
        self.t = np.asarray(t_grid)
        self.r0, self.s0 = r0, s0
        self.rng = rng

        # pre-compute Cholesky for [Z1, Z3]
        self._C = np.array([[1.0, 0.0],
                            [rho, np.sqrt(1 - rho**2)]])

    def simulate_paths(self, n_paths, store_paths=False):
        dt = np.diff(self.t)
        n_steps = len(dt)

        # allocate
        r = np.full((n_steps + 1, n_paths), self.r0)
        s = np.full((n_steps + 1, n_paths), self.s0)
        discount = np.ones(n_paths)

        a, sig_r, sig_s = self.a, self.sigma_r, self.sigma_s
        C = self._C

        for k, h in enumerate(dt):
            Z = self.rng.standard_normal(size=(2, n_paths))
            Z1, Z3 = (C @ Z)

            exp_ah = np.exp(-a * h)
            m = (self.theta(self.t[k + 1])
                 - sig_r**2 / (2 * a**2) * (1 - exp_ah)**2)

            # short rate step
            r[k + 1] = (r[k] * exp_ah
                        + m * (1 - exp_ah)
                        + sig_r * np.sqrt((1 - exp_ah**2) / (2 * a)) * Z1)

            # integral of r over the step
            I = ((1 - exp_ah) / a) * r[k] \
                + (h - (1 - exp_ah) / a) * m \
                + sig_r * np.sqrt((1 - exp_ah**2) / a**2) * Z1
            discount *= np.exp(-I)

            # asset step
            s[k + 1] = s[k] * np.exp(
                (r[k] - 0.5 * sig_s**2) * h
                + sig_s * np.sqrt(h) * Z3)

        return (r, s, discount) if store_paths else discount
