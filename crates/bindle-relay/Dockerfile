# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
#
# Bindle Cloud's relay. Build from the repository root:
#   docker build -f crates/bindle-relay/Dockerfile -t bindle-relay .
#   docker run -p 8080:8080 -v bindle-relay:/data bindle-relay
# It keeps only sealed data, in /data/relay.db; put HTTPS in front of it.

FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p bindle-relay

FROM debian:bookworm-slim
COPY --from=build /src/target/release/bindle-relay /usr/local/bin/bindle-relay
VOLUME /data
ENV PORT=8080
EXPOSE 8080
CMD ["bindle-relay", "--db", "/data/relay.db"]
