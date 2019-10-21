FROM rustlang/rust:nightly

COPY ./web/stream-plotter /app/web/stream-plotter

COPY "./build-web.sh" "/app/build-web.sh"

WORKDIR /app

RUN "./build-web.sh"

COPY "./src" "/app/src"

COPY "./Cargo.toml" "/app/Cargo.toml"

COPY "./Cargo.lock" "/app/Cargo.lock"

COPY "./build.sh" "/app/build.sh"

RUN "./build.sh"

COPY "./run.sh" "/app/run.sh"

CMD [ "./run.sh" ]
