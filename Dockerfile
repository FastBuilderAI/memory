FROM rust:1.80 as builder

WORKDIR /usr/src/fastmemory
COPY . .

# Build the binary
RUN cargo build --release --bin fastmemory

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/fastmemory/target/release/fastmemory /usr/local/bin/fastmemory

ENTRYPOINT ["fastmemory"]
