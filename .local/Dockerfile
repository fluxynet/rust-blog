# Stage 1: Build dependencies separately to cache them
FROM rust:1.85.0 AS chef
WORKDIR /app

# Install cargo-chef
RUN cargo install cargo-chef

# Copy only Cargo files to calculate dependencies
COPY Cargo.toml Cargo.lock ./

# Generate the recipe for caching dependencies
RUN cargo chef prepare --recipe recipe.json

# Build dependencies separately (cached)
RUN cargo chef cook --release --recipe recipe.json

# Stage 2: Development environment
FROM rust:1.85.0 AS dev
WORKDIR /app

# Install cargo-watch for live reloading
RUN cargo install cargo-watch

# Copy dependencies from the previous stage
COPY --from=chef /app/target target
COPY --from=chef /usr/local/cargo /usr/local/cargo

# Copy the full source code
COPY . .

# Start the application with auto-reloading
CMD ["cargo", "watch", "-x", "run"]
