# Multi-stage build for the agent-server binary.
FROM rust:1.84-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked -p agent-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agent-server /usr/local/bin/agent-server
# Firebase App Hosting injects PORT; default to 8080 for local runs.
ENV PORT=8080
EXPOSE 8080
CMD ["agent-server"]
