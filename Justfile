default:
    just --list

build:
    cargo build --all-features
    cargo build --target thumbv7m-none-eabi --no-default-features --features heapless

test:
    cargo test --all-features

clippy:
    cargo clippy --all-features
    cargo clippy --all-features --tests

check:
    cargo deny fetch
    cargo deny check
