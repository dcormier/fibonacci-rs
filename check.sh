set -x
cargo clippy
cargo clippy --all-features
cargo clippy --no-default-features
cargo clippy --tests
cargo clippy --tests --all-features
cargo clippy --tests --no-default-features
cargo test
cargo test --all-features
cargo test --no-default-features
# Benchmarks to be on nightly until this is stabalized:
# https://doc.rust-lang.org/nightly/unstable-book/library-features/test.html
cargo +nightly bench --features bench --profile release
cargo doc --all-features
set +x
