# Build the daemon
FROM rust:alpine AS buildenv
COPY ./ /buildroot
RUN apk add git
RUN cargo build --config=net.git-fetch-with-cli=true --release --manifest-path /buildroot/Cargo.toml


# Build the real container
FROM haproxy:alpine
COPY --from=buildenv /buildroot/target/release/haproxy_autoconfd /usr/local/bin/haproxy_autoconfd

USER root
EXPOSE 80
EXPOSE 443
CMD ["/usr/local/bin/haproxy_autoconfd"]
