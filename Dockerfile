FROM clux/muslrust:stable AS builder
WORKDIR /app

COPY . .

RUN cargo build --release

FROM scratch
COPY --from=builder /app/target .