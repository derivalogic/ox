# Ox

Ox is a Rust workspace containing several crates for quantitative finance and scripting.

## Crates

- **rustatlas** – financial analytics library used for pricing and market data utilities.
- **lefi** – lightweight scripting engine built on top of `rustatlas`.
- **edge_service** – experimental service layer integrating the libraries.

## Building

The workspace uses Cargo and requires a stable Rust toolchain.

```bash
cargo build --workspace
```

Running the test suite can be done with:

```bash
cargo test --workspace
```

## Scripting Example

The `lefi` crate allows evaluating small scripts. The snippet below shows how to
parse and evaluate a simple expression:

```rust
use lefi::prelude::*;
use lefi::utils::errors::Result;

fn main() -> Result<()> {
    let script = "
        x = 1;
        y = 2;
        z = x + y;
    ";

    // Parse the script into an expression tree
    let expr = ExprTree::try_from(script)?;

    // Index variables so the evaluator knows how many to create
    let indexer = EventIndexer::new();
    indexer.visit(&expr)?;

    // Evaluate the expression tree
    let evaluator = ExprEvaluator::new()
        .with_variables(indexer.get_variables_size());
    evaluator.const_visit(expr)?;

    println!("Variables: {:?}", evaluator.variables());
    Ok(())
}
```

Running this program prints:

```
Variables: [Number(1.0), Number(2.0), Number(3.0)]
```

See `scripting/examples` for runnable examples, including pricing scripts for a vanilla swap, barrier option, Asian option, forward, and a vanilla option.
