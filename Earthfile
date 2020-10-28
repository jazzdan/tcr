FROM rust:1.47
WORKDIR /usr/src/tcr

code:
    COPY Cargo.lock .
    COPY Cargo.toml .
    COPY src src

test:
    FROM +code
    RUN cargo test

build:
    FROM +code
    RUN cargo build --release
    RUN cargo install --path .
    SAVE ARTIFACT /usr/local/cargo/bin/tcr /tcr AS LOCAL build/tcr

docker:
    COPY +build/tcr .
    ENTRYPOINT ["/usr/src/tcr/tcr"]
    SAVE IMAGE tcr:latest

ci:
    BUILD +test
    BUILD +build

all:
    BUILD +test
    BUILD +build