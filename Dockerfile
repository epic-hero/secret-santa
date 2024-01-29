FROM docker.io/library/rust:1.67-alpine as builder
RUN apk add --no-cache musl-dev sqlite-static openssl-dev openssl-libs-static pkgconf git libpq-dev

ENV SYSROOT=/dummy

ENV SQLITE3_STATIC=1 SQLITE3_LIB_DIR=/usr/lib/

ENV LIBPQ_STATIC=1

ARG ELOXIDE_TOKEN
ARG DATABASE_URL
ARG RUST_LOG

WORKDIR /wd
COPY . /wd
RUN cargo build

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /wd/target/debug/bot /

#CMD ["./bot"]
