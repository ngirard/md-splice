# Convenient recipes for developing md-splice

# Default task: run all checks and tests
default: check test

# Run all tests
test:
    cargo test -- --nocapture

# Run tests and automatically re-run on file changes.
# Requires `cargo install cargo-watch`.
watch:
    cargo watch -x "test -- --nocapture"

# Check the project for compilation errors
check:
    cargo check

# Run clippy for linting and style checks
lint:
    cargo clippy -- -D warnings

# Run the application with given arguments.
# Example: `just run -- --file README.md --help`
run *args:
    cargo run -- {{args}}

# Review and accept/reject any snapshot changes from `insta`.
insta:
    cargo insta review

# Generate a directory snapshot for the project
snapshot:
    #!/usr/bin/env bash
    project_name="$(basename "${PWD%.git}")"
    snapshot_filename=".${project_name}_repo_snapshot.md"
    RIPGREP_CONFIG_PATH="{{invocation_directory()}}/.ripgreprc" dir2prompt > "${snapshot_filename}"
    wc -c "${snapshot_filename}"
