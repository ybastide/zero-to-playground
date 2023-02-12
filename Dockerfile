FROM rust:bookworm as base
RUN apt-get update && apt-get install -y mold clang

FROM base as planner
LABEL authors="zeb"
WORKDIR app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base as cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM base as builder

WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target

ENV SQLX_OFFLINE true
RUN cargo build --release
#RUN --mount=type=cache,target=/usr/local/cargo/registry \
#   	--mount=type=cache,target=/app/target \
#   	cargo build --release

FROM ubuntu:latest as runtime
WORKDIR app
COPY configuration configuration
COPY --from=builder /app/target/release/zero2prod zero2prod

ENV APP_ENVIRONMENT production
ENTRYPOINT ["/app/zero2prod"]
