FROM rust:latest as builder
WORKDIR /usr/src/outpack_server
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/outpack_server /usr/local/bin/outpack_server
COPY --from=builder /usr/src/outpack_server/Rocket.toml .
COPY start-with-wait .
EXPOSE 8000
ENTRYPOINT ["start-with-wait"]
