FROM rust:1.72 as builder
WORKDIR /usr/src/mosaic
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/mosaic /usr/local/bin/mosaic

CMD ["mosaic"]

EXPOSE 3030
