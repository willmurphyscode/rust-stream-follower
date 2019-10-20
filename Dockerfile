FROM rustlang/rust:nightly

COPY . /app

WORKDIR /app

RUN "./build.sh"

ENTRYPOINT [ "./target/release/rust-stream-follower" ]
