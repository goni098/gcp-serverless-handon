FROM rust:1.100-bookworm AS chef
ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
RUN cargo install cargo-chef --version 0.1.78 --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY pvault/Cargo.toml ./pvault/Cargo.toml
COPY pvault/clients/pvault ./pvault/clients/pvault
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json ./recipe.json
COPY --from=planner /app/pvault ./pvault
RUN cargo chef cook --release --workspace --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked --workspace

FROM gcr.io/distroless/cc-debian12:nonroot AS migration
WORKDIR /app
COPY --from=build /app/target/release/cli /usr/local/bin/cli
ENTRYPOINT ["/usr/local/bin/cli"]
CMD ["migration", "apply"]

FROM gcr.io/distroless/cc-debian12:nonroot AS function
ARG FUNCTION
COPY --from=build /app/target/release/${FUNCTION} /usr/local/bin/function
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/function"]
