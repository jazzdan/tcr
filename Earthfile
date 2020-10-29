FROM rust:1.47
WORKDIR /usr/src/tcr

deps-files:
    COPY Cargo.lock .
    COPY Cargo.toml .

deps:
    FROM +code
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

ci:
    BUILD +test
    BUILD +build

all:
    BUILD +test
    BUILD +build