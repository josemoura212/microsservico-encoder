FROM golang:1.25-alpine

ENV CGO_ENABLED=1

RUN apk add --no-cache \
    ffmpeg bash make gcc musl-dev sqlite-dev \
    wget unzip libc6-compat

ENV BENTO4_PATH="/opt/bento4"
RUN wget -q https://www.bok.net/Bento4/binaries/Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    unzip Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip && \
    mv Bento4-SDK-1-6-0-641.x86_64-unknown-linux ${BENTO4_PATH} && \
    rm Bento4-SDK-1-6-0-641.x86_64-unknown-linux.zip

ENV PATH="${BENTO4_PATH}/bin:${PATH}"

WORKDIR /go/src

ENTRYPOINT ["tail", "-f", "/dev/null"]
