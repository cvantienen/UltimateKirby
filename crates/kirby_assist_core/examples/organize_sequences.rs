//! Host-only formatter: stdin -> stdout, validated before and after layout.
use std::io::Read;
fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    kirby_assist_core::sequence_config::persist::validate_output(&source).unwrap();
    let output = kirby_assist_core::sequence_config::layout::organize(&source).unwrap();
    kirby_assist_core::sequence_config::persist::validate_output(&output).unwrap();
    print!("{output}");
}
