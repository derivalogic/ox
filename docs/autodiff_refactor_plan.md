# Autodiff Refactor Plan

This document outlines the remaining work required to complete the introduction of `T: Real` generics across the computational graph structures.  Several modules have been partially migrated while others are still concrete `f64` types.  Completing the refactor will allow using `f64` or the AD type `Var` interchangeably.

## 1. Structures still using `f64`

The following modules expose types with hard coded `f64` values.  They must become generic over `T: Real`.

- `cashflows`: `Cashflow`, `SimpleCashflow`, `FixedRateCoupon`, `FloatingRateCoupon`.
- `instruments`: all instrument structs (`FixedRateInstrument`, `FloatingRateInstrument`, `LoanDepo`, etc.) and builders (`Make*` helpers). **(done)**
- `alm` (asset/liability management) structs such as `CashAccount`, `PositionGenerator` and `RolloverSimulationEngine`.
- visitors expecting `f64` market data – e.g. `NPVConstVisitor` already generic but some visitors and helpers still assume `f64`. **(in progress)**
- any pricing models still using `f64` (check models under `models` folder).

Each struct should take a type parameter `<T: Real>` and store numeric fields as `T` instead of `f64`.  Associated methods and traits should be updated accordingly.  Builders should also be generic and default to `f64` for ergonomic use.

## 2. Trait updates

Traits that operate on these structs (e.g. `Payable`, `HasCashflows`, `Model`) need generic versions.  Many already accept `T: Real`, but some methods still return `f64`.  Ensure all numeric returns use `T`.

Example to change:
```rust
pub trait Payable {
    fn amount(&self) -> Result<f64>; // replace with Result<T>
}
```

## 3. Integration with AD `Var`

After generics are in place, add unit tests demonstrating that the same API works with both `f64` and `Var` from `rustatlas::math::ad`.

### Doctest snippet
```rust
use rustatlas::math::ad::{self, Var};
use rustatlas::prelude::*;

let cf = SimpleCashflow::<Var>::new(Date::new(2025,1,1), Currency::USD, Side::Receive)
    .with_amount(Var::from(100.0));
assert_eq!(cf.amount().unwrap().value(), 100.0);
```
This should compile once generics are implemented.

## 4. Additional tests

- Convert existing module tests to instantiate types with `Var` in addition to `f64`.
- Add regression tests for builders (e.g. `MakeFixedRateInstrument::<Var>`).
- Provide doctests for public examples in `rustatlas/README.md` using the generic form.

## 5. Gradual migration strategy

1. Start with low level cashflow structs.
2. Propagate generics to instrument structs and visitors.
3. Update builders and model interfaces.
4. Finally refactor examples and tests.

Tracking progress in this document will help ensure full coverage before enabling AD throughout the library.
