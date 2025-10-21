//! The md-splice command-line executable.

fn main() -> anyhow::Result<()> {
    // By calling the library's run function, we keep the binary crate minimal.
    // This is good practice for testing and code organization.
    md_splice_lib::run()
}
