# rust-stream-follower
Learn to parse a high-volume stream of data by connecting a client written in Rust to the 
Twitter streaming API and showing (very) simple sentiment analysis in a WASM HTML canvas
chart.

# Online Version

You can see the running app [here](https://pacific-wave-78223.herokuapp.com/)

# Running Locally

Running this project locally in Docker requires Docker and Twitter API credentials.
Running it locally on the host requires the Rust nightly toolchain with compilers targeting
the host and targeting wasm32-unknown-unknown, and the wasm-bindgen CLI.

## To Run in Docker:

```bash
export API_KEY="$YOUR_CONSUMER_KEY"
export API_SECRET="$YOUR_CONSUMER_SECRET"
export ACCESS_TOKEN="$YOUR_ACCESS_KEY"
export ACCESS_SECRET="$YOUR_ACCESS_SECRET"
PORT="3000" ./run-docker.sh
```

## To Run Outside Docker

1. Export the same variables as above.
2. Run build-web.sh
3. Run build.sh
4. Run `PORT="3000" ./run.sh`

# Architecture

This project compiles a single binary (or single Docker image to start that binary), that
spawns a number of threads:

1. One thread connects to the twitter streaming API and follows new tweets for a hard-coded set of keywords.
   This thread sends every new tweet on a channel to thread 2.
2. This thread receives tweets and updates an in memory datastructure that represents, for each keyword, how many tweets have seemed happy, sad, or neutral.
3. The third thread (which likely spawns more) creates a Rocket API server exposing the following routes:
    - `/` - Get an `index.html` that loads the front-end
    - `/current` - Get the current count of tweeets by keyword and sentiment
    -  Two routes that serve the compiled JS and WebAssembly that draw the front-end

# Known Issues

## Front-End

1. Front-end hackiness - The Rust library that renders the chart on the HTML canvas has proven challenging to customize. The chart still looks a little weird, and parts of it feel hacked together.
1. Axis scales - these are wrong because I haven't figured out how to make [Plotters](https://github.com/38/plotters) do what I want.
1. Front-end scaling - after a few thousand tweeets, the bars on the graph are too large to display properly.
1. Hard-coded pixel scale - There are hard-coded pixel offsets in the front-end code that I arrived at by guessing and checking. This mess should be replaced with a better layout.
1. Not responsive - as a direct consequence of the item above, the chart will always be 600 x 500 pixels, and doesn't scale well for all devices.

## Back-End

The back-end is working as designed, but there are two things that, given enough time, I would want to add:

1. A load-testing server - this would be a simple HTTP server that would imitate Twitter's streaming API but just send a hard-coded array of tweets in a tight loop forever. This load-testing server would let me measure how high a volume of tweet this code can keep up with.
1. A load-measuring client - this would be an HTTP client that would connect to Twitter's Streaming API and read tweets as quickly as possible, by just discarding all the data and sending back `Ack`s as fast as it can. This would help me ensure that I'm not putting back-pressure on my connection to the streaming API be acknowledging packets too slowly.
1. Inability to horizontally scale - This back-end opens a single connection to the Twitter streaming API, and exposes summary data to N clients. However, the summary data is an in-memory data-structure, meaning every instance serving the front-end needs its own connection to the Twitter stream, and will present different results. Putting the summary data structure in some fast network attached cache would allow horizontally scaling the app.

## Development

1. Dockerfile always builds release mode - this means that compilation times are higher. The build scripts should be updated so that release vs. debug builds are configurable.
1. Dockerfile not good for incremental compilation - local development in Rust benefits from incremental compilation. That is, the Rust compiler will try to only rebuild what is has to. However, the current structure of the Dockerfile throws away intermediate build state, so we pay the full compile-time every time. I should explore how to fix this.