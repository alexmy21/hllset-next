# HLLSet Algebra — Redis container with modules
# Build: podman build -t hllset-redis -f redis/Dockerfile .
# Run:   podman-compose -f redis/docker-compose.yml up -d
FROM redis:7.0.15

RUN apt-get update && apt-get install -y libgomp1 && rm -rf /var/lib/apt/lists/*

# Copy pre-built Redis modules (all processing is in Rust, Redis is storage only)
COPY redis/modules/libredis-roaring.so /usr/local/lib/redis/
COPY redis/modules/redisearch.so /usr/local/lib/redis/
COPY redis/modules/redisgraph.so /usr/local/lib/redis/

# Copy Redis configuration
COPY redis/redis.conf /usr/local/etc/redis/redis.conf

EXPOSE 6379

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD redis-cli ping | grep PONG || exit 1

CMD ["redis-server", "/usr/local/etc/redis/redis.conf"]
