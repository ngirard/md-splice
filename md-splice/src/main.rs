//! The md-splice command-line executable.

mod app;
mod cli;

fn main() -> anyhow::Result<()> {
    app::run()
}
