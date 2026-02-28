FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache \
    musl-dev ffmpeg bash make python3 pkgconfig \
    openssl-dev openssl-libs-static gcc \
    wget unzip libc6-compat libstdc++ libgcc

ENV BENTO4_PATH="/opt/bento4"
WORKDIR /tmp/bento4
RUN wget -q https://www.bok.net/Bento4/binaries/Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    unzip Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux ${BENTO4_PATH} && \
    rm Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src/ src/
COPY migrations/ migrations/
COPY .sqlx/ .sqlx/

ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM alpine:3.21
RUN apk add --no-cache ca-certificates ffmpeg bash libc6-compat libstdc++ libgcc python3

COPY --from=builder /opt/bento4 /opt/bento4
ENV PATH="/opt/bento4/bin:${PATH}"

COPY --from=builder /usr/src/app/target/release/encoder-rust /usr/local/bin/encoder-rust

EXPOSE 8080
CMD ["encoder-rust"]
