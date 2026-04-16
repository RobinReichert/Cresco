echo 'Running cargo fmt...' >&2
cargo fmt --check

echo 'Running cargo test...' >&2
CARGO_TERM_QUIET=true cargo test -p logic

echo 'Running cargo check...' >&2
cd cross
CARGO_TERM_QUIET=true cargo check --release
cd ..


