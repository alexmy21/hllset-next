#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "Building and deploying HLLSet Redis stack..."
bash redis/build.sh
podman-compose -f redis/docker-compose.yml up -d --build
echo "Redis stack running on port 6379"
echo "Modules: redis-roaring, redisearch, redisgraph, redis-hllset"
