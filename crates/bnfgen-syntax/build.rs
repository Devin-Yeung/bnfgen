use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Cargo reruns the build script when the grammar changes; process_src
    // is idempotent, so this is all the wiring the crate needs.
    println!("cargo:rerun-if-changed=src/parser.lalrpop");
    lalrpop::process_src()?;
    Ok(())
}
