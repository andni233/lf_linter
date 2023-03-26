FROM rust:1.68 as builder

WORKDIR /usr/src/lf_lint
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/lf_lint /usr/local/bin/lf_lint
ENTRYPOINT ["lf_lint"]
