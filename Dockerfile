# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build
WORKDIR /usr/src/moodlescraper
COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/moodlescraper*
COPY . .
RUN cargo build --release
RUN cargo install --path .
# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:latest
RUN apt-get update && apt-get -y install openssl
COPY --from=cargo-build /usr/local/cargo/bin/moodlescraper /usr/local/bin/moodlescraper
CMD ["moodlescraper"]