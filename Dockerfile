FROM ekidd/rust-musl-builder:1.41.0 as build

WORKDIR /usr/src/app

COPY . .

RUN sudo chown -R rust:rust .

RUN cargo build --target=x86_64-unknown-linux-musl

# 
# Some other hacky way would be pre compiling cross platform and
# build the base straight without multi stage docker
#
# Pre-image build Command:
# rustup target install x86_64-unknown-linux-musl
# cargo install cross
# build --target=x86_64-unknown-linux-musl
#

FROM alpine:3.9.5

WORKDIR /app

COPY --from=build /usr/src/app/target/x86_64-unknown-linux-musl/debug/pinger-rs /app/bin/pinger-rs
COPY config.yml /app/config/config.yml

EXPOSE 9090

ENTRYPOINT ["/app/bin/pinger-rs"]
CMD ["/app/config/config.yml"]
