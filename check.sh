cargo clippy && \
cargo clippy --all-features && \
cargo clippy --no-default-features && \
cargo clippy --tests && \
cargo clippy --tests --all-features && \
cargo clippy --tests --no-default-features && \
cargo test && \
cargo test --all-features && \
cargo test --no-default-features && \
cargo doc --all-features
