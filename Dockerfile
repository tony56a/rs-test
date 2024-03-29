# Build image
FROM rust:1.61-buster as build-image

WORKDIR /usr/src/app

COPY . .

ARG CI_SHA1

RUN apt-get update && apt-get install --no-install-recommends \
 libopus-dev \
 -y

RUN echo $CI_SHA1
RUN CIRCLE_SHA1=$CI_SHA1 cargo build --release

# deployment image

FROM debian:buster-slim

RUN apt-get update && apt-get install --no-install-recommends \
 libopus-dev \
 ffmpeg \
 ca-certificates \
 python3 \
 python3-pip \
 espeak \
 -y

RUN pip3 install youtube-dl

COPY resources resources
COPY --from=build-image /usr/src/app/target/release/rs-test /usr/local/bin/rs-test

EXPOSE 3030

CMD ["rs-test"]


