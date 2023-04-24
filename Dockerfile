FROM rustlang/rust:nightly-buster-slim as builder
# FROM rust:1.68 as builder
WORKDIR /build
RUN echo "aaa3"
RUN pwd
COPY . .
RUN apt-get update
RUN apt-get install -y binaryen
RUN apt-get install libssl-dev pkgconf make -y
# RUN rustup default nightly
# RUN rustup update nightly 
RUN rustup target add wasm32-unknown-unknown

RUN rustup component list --installed


# RUN rustup show
# RUN rustup toolchain remove nightly-x86_64-unknown-linux-gnu
# RUN rustup install nightly-x86_64-unknown-linux-gnu

# RUN rustup target add wasm32-unknown-unknown
# RUN rustup component add --toolchain nightly rust-std

#RUN cargo install vertigo-cli

RUN rustup target list

RUN cargo install --git https://github.com/vertigo-web/vertigo --branch cli vertigo-cli 

RUN vertigo build --dest-dir=./build vertigo-demo
RUN rm -Rf build_dist
RUN mkdir build_dist
RUN cp -R build build_dist/build
RUN cargo build --bin server
RUN cp target/debug/server build_dist

#RUN ./build.sh


FROM debian:buster-slim
RUN apt update && apt install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*
RUN mkdir /app
WORKDIR /app
COPY --from=builder /monitor/build_dist /app
CMD ["./server"]
