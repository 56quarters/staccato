FROM scratch
COPY target/x86_64-unknown-linux-musl/release/st /bin/st
CMD ["/bin/st", "--help"]
