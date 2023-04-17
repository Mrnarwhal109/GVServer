# We use the latest Rust stable release as base image
# FROM rust:1.67.1 AS builder
FROM lukemathwalker/cargo-chef:latest-rust-1.67.1 AS chef
# Let's switch our working directory to 'app' (equivalent to 'cd app')
# The 'app' folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

FROM chef AS planner
# Copy all files from our working environment to our Docker image
COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to the point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it faaaaaaaaaaast
RUN cargo build --release --bin gv_server

# Runtime stage
#FROM rust:1.67.1-slim AS runtime
FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
# Clean up
&& apt-get autoremove -y \
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/gv_server gv_server
# We need the configuration file at runtime!
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENV RUST_BACKTRACE 1

# When 'docker run' is executed, launch the binary!
ENTRYPOINT ["./gv_server"]
