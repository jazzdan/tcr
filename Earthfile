FROM rust:1.47
WORKDIR /usr/src/tcr

build:
    COPY Cargo.lock .
    COPY Cargo.toml .
    COPY src src
    RUN cargo build --release
    RUN cargo install --path .
    SAVE ARTIFACT /usr/local/cargo/bin/tcr /tcr AS LOCAL build/tcr

docker:
    COPY +build/tcr .
    ENTRYPOINT ["/usr/src/tcr/tcr"]
    SAVE IMAGE tcr:latest