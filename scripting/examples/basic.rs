use lefi::prelude::*;
use lefi::utils::errors::Result;

fn main() -> Result<()> {
    let script = "
        x = 1;
        y = 2;
        z = x + y;
    ";

    let expr = ExprTree::try_from(script)?;

    let indexer = EventIndexer::new();
    indexer.visit(&expr)?;

    let evaluator = ExprEvaluator::new()
        .with_variables(indexer.get_variables_size());
    evaluator.const_visit(expr)?;

    println!("Variables: {:?}", evaluator.variables());
    Ok(())
}
