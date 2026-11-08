FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo test --release --locked && cargo build --release --locked && cp target/release/todo-server /app/todo-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* && useradd --system --create-home --uid 10001 app
ENV HOME=/home/app
WORKDIR /app
COPY --from=build /app/todo-server /usr/local/bin/todo-server
COPY certs ./certs
USER app
EXPOSE 8080
CMD ["todo-server"]
