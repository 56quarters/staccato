FROM alpine:latest
WORKDIR /build
COPY target/x86_64-unknown-linux-musl/release/st /build/
RUN install -m 755 /build/st /usr/local/bin/st && rm -r /build
CMD ["st", "--help"]
