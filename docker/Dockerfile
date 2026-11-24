FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
ARG CARGO_BUILD_JOBS
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json ./recipe.json
RUN cargo chef cook --release --workspace --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked --workspace

FROM gcr.io/distroless/cc-debian12:nonroot AS migration
WORKDIR /app
COPY --from=build /app/target/release/migration /usr/local/bin/migration
ENTRYPOINT ["/usr/local/bin/migration"]
CMD ["up"]

FROM gcr.io/distroless/cc-debian12:nonroot AS function
ARG FUNCTION
COPY --from=build /app/target/release/${FUNCTION} /usr/local/bin/function
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/function"]
