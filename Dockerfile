FROM rust:latest as builder
WORKDIR /usr/src/outpack_server
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/outpack_server /usr/local/bin/outpack_server
CMD ["outpack_server", "--root", "/outpack"]
