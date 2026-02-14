FROM rust:1.85-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release -p kamn-node

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends curl ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --uid 10001 kamn

COPY --from=builder /app/target/release/kamn-node /usr/local/bin/kamn-node

USER kamn
WORKDIR /home/kamn

ENTRYPOINT ["/usr/local/bin/kamn-node"]
CMD ["--role", "processor", "--runtime-mode", "daemon", "--daemon-max-ticks", "3", "--daemon-tick-interval-ms", "25", "--output", "json"]
