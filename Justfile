default:
    just --list

build:
    cargo build --all-features

test:
    cargo test --all-features

clippy:
    cargo clippy --all-features
    cargo clippy --all-features --tests
