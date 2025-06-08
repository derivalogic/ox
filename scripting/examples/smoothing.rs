// use scripting::prelude::*;
// use scripting::utils::errors::Result;
// use scripting::visitors::evaluator::{SingleScenarioEvaluator, Value};

// fn eval_payoff(script: &str) -> Result<f64> {
//     let expr = ExprTree::try_from(script.to_string())?;
//     let indexer = EventIndexer::new();
//     indexer.visit(&expr)?;
//     let var_map = indexer.get_variable_indexes();
//     let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
//     evaluator.const_visit(expr)?;
//     let vars = evaluator.variables();
//     if let Some(idx) = var_map.get("payoff") {
//         if let Some(Value::Number(n)) = vars.get(*idx) {
//             return Ok(n.value());
//         }
//     }
//     Err(ScriptingError::EvaluationError(
//         "payoff not found".to_string(),
//     ))
// }

// fn main() -> Result<()> {
//     let if_script = "
//         spot = 100;
//         payoff = 0;
//         if spot > 100 {
//             payoff = 1;
//         } else {
//             payoff = 0;
//         }
//     ";

//     let fif_script = "
//         spot = 100;
//         payoff = fif(spot - 100, 1, 0, 1);
//     ";

//     let regular = eval_payoff(if_script)?;
//     let fuzzy = eval_payoff(fif_script)?;

//     println!("Payoff with regular if: {}", regular);
//     println!("Payoff with fif: {}", fuzzy);
//     Ok(())
// }

fn main() {}