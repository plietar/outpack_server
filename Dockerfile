FROM rust:latest as builder
WORKDIR /usr/src/outpack_server
COPY . .
RUN echo "nameserver 8.8.8.8" > /etc/resolv.conf && cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/outpack_server /usr/local/bin/outpack_server
CMD ["outpack_server", "--root", "/outpack"]
