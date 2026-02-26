FROM rust:1.93.1 AS builder
WORKDIR /app
ARG CORTEX_CORE_READ_TOKEN
COPY . .
RUN if [ -n "$CORTEX_CORE_READ_TOKEN" ]; then \
      git config --global url."https://x-access-token:${CORTEX_CORE_READ_TOKEN}@github.com/".insteadOf "https://github.com/"; \
    fi
RUN cargo build --locked -p cortex-app --bin cortex --release && \
    cargo build --locked -p rmvm-sidecar --bin rmvm-grpc-server --release

FROM debian:bookworm-slim
RUN useradd -m -u 10001 cortex
WORKDIR /home/cortex
COPY --from=builder /app/target/release/cortex /usr/local/bin/cortex
COPY --from=builder /app/target/release/rmvm-grpc-server /usr/local/bin/rmvm-grpc-server
USER cortex
ENV CORTEX_ENDPOINT=grpc://127.0.0.1:50051
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cortex"]
CMD ["proxy", "serve", "--addr", "0.0.0.0:8080"]
