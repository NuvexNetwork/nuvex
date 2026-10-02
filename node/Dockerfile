FROM rust:bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml ./
COPY programs programs
COPY crates crates
COPY node node
COPY sdk sdk
COPY cli cli
COPY tests tests
RUN cargo build --release -p nuvex-oracle-node

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 nuvex
USER nuvex
COPY --from=build /src/target/release/nuvex-node /usr/local/bin/nuvex-node
EXPOSE 8081
ENTRYPOINT ["nuvex-node"]
