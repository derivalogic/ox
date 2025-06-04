use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load `data.json` located next to this example
    let path = Path::new("data.json");
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    // Execute the pricing routine exposed by the wasm crate
    let result = wasm::run_simulation(&json).expect("simulation failed");

    // `run_simulation` returns a JsValue containing a JSON string
    let output = result
        .as_string()
        .unwrap_or_else(|| "<non-string result>".to_string());
    println!("{output}");

    Ok(())
}
