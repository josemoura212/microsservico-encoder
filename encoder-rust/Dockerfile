FROM rustlang/rust:nightly-alpine AS builder

# Install dependencies
RUN apk add --no-cache musl-dev ffmpeg bash make python3 pkgconfig openssl-dev openssl-libs-static gcc libc6-compat libstdc++ libgcc

# Install Bento4
WORKDIR /tmp/bento4
ENV BENTO4_BASE_URL="https://www.bok.net/Bento4/binaries/" \
    BENTO4_BINARY="Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip" \
    BENTO4_PATH="/opt/bento4"

RUN apk add --no-cache bash wget unzip && \
    wget -q ${BENTO4_BASE_URL}/${BENTO4_BINARY} && \
    unzip ${BENTO4_BINARY} && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux ${BENTO4_PATH} && \
    rm ${BENTO4_BINARY}

# Set working directory
WORKDIR /usr/src/app

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

# Copy sqlx offline data for compile-time verification
COPY .sqlx/ .sqlx/

# Set environment variable for sqlx offline mode
ENV SQLX_OFFLINE=true

# Build the application
RUN cargo build --release

# Final stage
FROM alpine:latest
RUN apk add --no-cache ca-certificates ffmpeg bash make gcc musl-dev libc6-compat libstdc++ libgcc

# Copy Bento4
COPY --from=builder /opt/bento4 /opt/bento4
ENV PATH="$PATH:/opt/bento4/bin"

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/encoder-rust /usr/local/bin/encoder-rust

# Expose port if needed
EXPOSE 8080

# Run the application
CMD ["encoder-rust"]
