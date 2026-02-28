FROM golang:1.25-alpine

RUN apk add --no-cache \
    bash curl wget unzip git \
    ffmpeg python3 \
    make gcc musl-dev sqlite-dev pkgconfig \
    openssl-dev openssl-libs-static \
    libc6-compat libstdc++ libgcc

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

ENV BENTO4_PATH="/opt/bento4"
RUN wget -q https://www.bok.net/Bento4/binaries/Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    unzip Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux ${BENTO4_PATH} && \
    rm Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip

ENV PATH="${BENTO4_PATH}/bin:${PATH}" \
    CGO_ENABLED=1

RUN go install golang.org/x/tools/gopls@latest && \
    cargo install sqlx-cli --no-default-features --features postgres,sqlite

WORKDIR /workspace
