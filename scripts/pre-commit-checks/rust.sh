echo 'Running cargo fmt...' >&2
cargo fmt --check
echo 'Running cargo test...' >&2
cargo test -p logic
echo 'Running cargo check...' >&2
cd cross
cargo check --release
cd ..


