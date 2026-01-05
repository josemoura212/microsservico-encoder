FROM golang:1.25-alpine
ENV PATH="$PATH:/bin/bash" \
    BENTO4_BIN="/opt/bento4/bin" \
    PATH="$PATH:/opt/bento4/bin" \
    CGO_ENABLED=1

# FFMPEG and build dependencies for CGO
RUN apk add --update ffmpeg bash make gcc musl-dev sqlite-dev

# Install glibc compatibility for Bento4 binaries
RUN apk add --no-cache libc6-compat

# Install Bento4 from binary
RUN apk add --update --upgrade bash && \
    wget -q https://www.bok.net/Bento4/binaries/Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    unzip "Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip" && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux /opt/bento4 && \
    rm "Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip"

WORKDIR /go/src

ENTRYPOINT ["tail", "-f", "/dev/null"]
