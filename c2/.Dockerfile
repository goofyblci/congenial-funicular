FROM rust:latest AS build

WORKDIR /c2

COPY . ./

# Build the app
RUN cargo build --release

FROM debian:bookworm-slim AS c2

RUN apt-get update
RUN apt-get install -y tor supervisor 

COPY --from=build /c2/target/release/c2 /

CMD ["/c2"]