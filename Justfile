# compile the binary and run unit tests
all: build test

# compile the binary and run unit tests (in release mode)
all-release: build-release test-release

# check coding style and lint code
quality: fmt check clippy

# compile the binary
@build:
    cargo build

# compile the binary (in release mode)
@build-release:
    cargo build --release --verbose

# run unit tests
@test:
    cargo test --workspace -- --quiet

# run unit tests (in release mode)
@test-release:
    cargo test --workspace --release --verbose

# format code
@fmt:
    cargo fmt

# check code for error
@check:
    cargo check

# lint
@clippy:
    cargo clippy
