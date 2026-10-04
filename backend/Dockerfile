FROM rust:1.99-slim-bookworm AS builder
WORKDIR /app

# The utoipa-swagger-ui build script downloads the UI assets with curl.
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN adduser book && chown -R book /app
USER book
COPY --from=builder ./app/target/release/app ./target/release/app

ENV PORT=8080
EXPOSE ${PORT}
ENTRYPOINT [ "./target/release/app" ]
