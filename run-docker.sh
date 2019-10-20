#!/usr/bin/env bash

set -u

docker run -t --env API_KEY="$API_KEY" --env API_SECRET="$API_SECRET" \
    --env ACCESS_TOKEN="$ACCESS_TOKEN" \
    --env ACCESS_SECRET="$ACCESS_SECRET" \
    --env ROCKET_PORT="$PORT" \
    --expose "$PORT" \
    -p "$PORT:$PORT" \
    streamy