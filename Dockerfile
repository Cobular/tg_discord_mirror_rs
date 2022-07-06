FROM rust:slim-buster AS chef 
RUN apt-get update && apt-get install -y pkg-config libssh-dev cmake
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
RUN mkdir /app
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update \
 && apt-get install --no-install-recommends --yes libssh-dev \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tg_discord_mirror /usr/local/bin
ENTRYPOINT ["/usr/local/bin/tg_discord_mirror"]