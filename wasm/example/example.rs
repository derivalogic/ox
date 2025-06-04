pub fn main() {
    use serde_json::Value;
    ///load json from current directory
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    let path = Path::new("data.json");
    let mut file = File::open(&path).expect("Unable to open file");
}
