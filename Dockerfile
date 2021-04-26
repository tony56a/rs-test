# Build image
FROM rust:1.51-buster as build-image

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/myapp*

COPY . .

RUN apt-get update && apt-get install --no-install-recommends \
 libopus-dev \
 ffmpeg \
 -y


RUN cargo build --release

# deployment image

FROM debian:buster-slim

RUN apt-get update && apt-get install --no-install-recommends \
 libopus-dev \
 ffmpeg \
 ca-certificates \
 python3 \
 python3-pip \
 -y

RUN pip3 install youtube-dl

COPY --from=build-image /usr/src/app/target/release/rs-test /usr/local/bin/rs-test

EXPOSE 3030

CMD ["rs-test"]


