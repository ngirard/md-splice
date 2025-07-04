//! Core library for md-splice, containing all logic for AST manipulation.

pub mod cli;
pub mod error;
pub mod locator;
pub mod splicer;

/// The main entry point for the application logic.
pub fn run() -> anyhow::Result<()> {
    // TODO: Implement the main logic loop:
    // 1. Parse CLI args using cli::Cli::parse()
    // 2. Initialize logger
    // 3. Read input file
    // 4. Parse markdown to AST
    // 5. Locate target node
    // 6. Splice/modify AST
    // 7. Render AST to string
    // 8. Write to output (in-place or new file)
    println!("md-splice is running!");
    Ok(())
}
