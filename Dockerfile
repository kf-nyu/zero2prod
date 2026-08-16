#! Dockerfile
# chef
FROM lukemathwalker/cargo-chef:latest-rust-1.97.1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . . 
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

#builder stage
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Copy all files from our working environment to our Docker image
COPY . . 
# Let's build our binary!
# We'll use the release profile to make it fast
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM debian:bookworm-slim AS runtime 

# Let's switch our working directory to 'app' (equivalet to 'cd app')
# The 'app' folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
# Let's build our binary!
# We'll use the release profile to make it fast
COPY configuration configuration
ENV APP_ENVIRONMENT=production
# When 'docker run' is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]
