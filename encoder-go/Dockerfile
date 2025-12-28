FROM golang:1.21-alpine
ENV PATH="$PATH:/bin/bash" \
    BENTO4_BIN="/opt/bento4/bin" \
    PATH="$PATH:/opt/bento4/bin"

# FFMPEG
RUN apk add --update ffmpeg bash make

# Install Bento4 from binary
RUN apk add --update --upgrade bash && \
    wget -q https://www.bok.net/Bento4/binaries/Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    unzip "Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip" && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux /opt/bento4 && \
    rm "Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip"

WORKDIR /go/src

ENTRYPOINT ["tail", "-f", "/dev/null"]
