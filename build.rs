use std::process::Command;

fn main() {
    Command::new("cargo")
        .args(vec!["+nightly", "fmt"])
        .status()
        .expect("Failed to run `cargo fmt`.");
}
