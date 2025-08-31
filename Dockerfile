FROM rust:1.87.0 AS rust-builder
WORKDIR /app
COPY Cargo.toml /app/
COPY src /app/src/
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=rust-builder /app/target/release/pokebargainsjapan /app/pokebargainsjapan
COPY templates /app/templates
COPY static /app/static
COPY schema.sql /app/schema.sql
COPY Config.toml /app/Config.toml
RUN apt-get update
RUN apt-get -y install libssl3
EXPOSE 8000
ENTRYPOINT ["/app/pokebargainsjapan"]
