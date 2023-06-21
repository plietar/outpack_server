FROM rust:latest as builder
WORKDIR /usr/src/outpack_server
COPY . .
RUN cargo install --path .

FROM debian:buster-slim

RUN  apt-get -yq update && \
     apt-get -yqq install openssh-client

COPY --from=builder /usr/local/cargo/bin/outpack_server /usr/local/bin/outpack_server
COPY --from=builder /usr/src/outpack_server/Rocket.toml .
COPY start-with-wait /usr/local/bin
EXPOSE 8000
ENTRYPOINT ["start-with-wait"]
