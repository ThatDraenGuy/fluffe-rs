FROM rust:alpine as build-env
WORKDIR /app
COPY . /app
RUN apk add --no-cache musl-dev
RUN apk add --no-cache pkgconfig
RUN rustup update
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17
COPY --from=build-env /app/target/x86_64-unknown-linux-musl/release/fluffe-rs /
RUN chmod +x fluffe-rs
CMD ["./fluffe-rs"]