FROM rustlang/rust:nightly-slim AS build

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev

COPY . /vertigo-src

WORKDIR /vertigo-src

RUN cargo build --release -p vertigo-cli

FROM debian:stable-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /vertigo-src/target/release/vertigo .

CMD ["./vertigo"]
