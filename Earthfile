FROM rust:1.47
WORKDIR /usr/src/tcr

deps-files:
    COPY Cargo.lock .
    COPY Cargo.toml .

deps:
    FROM +code
    RUN rustup component add clippy --toolchain 1.47.0-x86_64-unknown-linux-gnu
    RUN cargo build --locked --release
    RUN rm -rf target/release/deps/tcr*
    SAVE ARTIFACT target target

code:
    FROM +deps-files
    COPY src src

test:
    FROM +code
    COPY +deps/target target
    RUN cargo test --locked

build:
    FROM +code
    COPY +deps/target target
    RUN cargo build --locked --release
    RUN cargo --locked install --path .
    SAVE ARTIFACT /usr/local/cargo/bin/tcr /tcr AS LOCAL build/tcr

docker:
    COPY +build/tcr .
    ENTRYPOINT ["/usr/src/tcr/tcr"]
    SAVE IMAGE tcr:latest

fmt:
    FROM +code
    RUN rustup component add rustfmt --toolchain 1.47.0-x86_64-unknown-linux-gnu
    RUN cargo fmt --all -- --check

clippy:
    FROM +deps
    RUN cargo clippy --all-targets --all-features

ci:
    BUILD +fmt
    BUILD +clippy
    BUILD +test
    BUILD +build

ci-macos:
    BUILD +test

all:
    BUILD +fmt
    BUILD +clippy
    BUILD +test
    BUILD +build
    BUILD +docker