FROM rust:1.81-alpine@sha256:d6e876ca5fe200f4ac60312b95606f0b042699c4cf6a19493b7d2a2ebbfb337b AS build
RUN apk update && apk add protoc protobuf-dev build-base && rm -rf /var/cache/apk/*
ENV TARGET_LIBC=musl TARGET_VENDOR=unknown

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin webhook
RUN mv ./target/release/webhook /usr/sbin/protect-webhook

FROM scratch
ENTRYPOINT ["/usr/sbin/protect-webhook"]
COPY --from=build /usr/sbin/protect-webhook /usr/sbin/protect-webhook
