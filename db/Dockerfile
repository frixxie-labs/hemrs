FROM postgres:18

# The base image already ships the PGDG apt repository, which provides PostGIS.
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates curl gnupg; \
    . /etc/os-release; \
    curl -fsSL https://packagecloud.io/timescale/timescaledb/gpgkey \
        | gpg --dearmor -o /usr/share/keyrings/timescaledb.gpg; \
    echo "deb [signed-by=/usr/share/keyrings/timescaledb.gpg] https://packagecloud.io/timescale/timescaledb/debian/ ${VERSION_CODENAME} main" \
        > /etc/apt/sources.list.d/timescaledb.list; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        "timescaledb-2-postgresql-${PG_MAJOR}" \
        "postgresql-${PG_MAJOR}-postgis-3"; \
    apt-get purge -y --auto-remove curl gnupg; \
    rm -rf /var/lib/apt/lists/*

CMD ["postgres", "-c", "shared_preload_libraries=timescaledb", "-c", "timescaledb.telemetry_level=off"]
