# hemrs (Home Environment Monitor-rs)

A high-performance IoT sensor monitoring backend built with Rust, designed to
collect, store, and serve environmental sensor data from distributed devices.

## Overview

hemrs provides a RESTful API for managing IoT devices and sensors, with
efficient data ingestion and querying capabilities. The system is optimized for
high-throughput measurement collection while maintaining low-latency queries
through caching and background processing.

## Architecture

### Core Components

- **HTTP Server**: Axum-based REST API server handling all client requests
- **Database Layer**: PostgreSQL with SQLx for type-safe database operations
- **Background Tasks**: Asynchronous measurement ingestion and view maintenance
- **Caching Layer**: Moka-based in-memory cache for recent measurements (60s
  TTL)
- **Metrics**: Prometheus integration for system monitoring and performance
  tracking

### Data Flow

1. **Ingestion**: Measurements are received via HTTP POST and queued for
   background processing
2. **Processing**: Background thread handles database insertion asynchronously
3. **Caching**: Recent measurements are cached for fast retrieval
4. **Querying**: REST endpoints serve data from cache or database with fallback

## Database Schema

### Tables

#### devices

```sql
CREATE TABLE devices(
    id SERIAL UNIQUE NOT NULL,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    PRIMARY KEY(id)
);
```

#### sensors

```sql
CREATE TABLE sensors(
    id SERIAL UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,
    unit TEXT NOT NULL,
    PRIMARY KEY(id)
);
```

#### measurements

```sql
CREATE TABLE measurements(
    id SERIAL UNIQUE NOT NULL,
    ts TIMESTAMP with time zone NOT NULL,
    device_id SERIAL NOT NULL REFERENCES devices (id),
    sensor_id SERIAL NOT NULL REFERENCES sensors (id),
    value REAL NOT NULL,
    PRIMARY KEY (id)
);
```

### Views

#### device_sensors (Materialized View)

Links devices to sensors they've reported measurements for:

```sql
CREATE MATERIALIZED VIEW device_sensors AS
SELECT m.device_id, s.id as sensor_id, s.name, s.unit
FROM measurements m
JOIN sensors s ON m.sensor_id = s.id
GROUP BY (m.device_id, s.id)
ORDER BY m.device_id;
```

## API Endpoints

### Devices Management

- `GET /api/devices` - List all devices
- `POST /api/devices` - Create new device
- `PUT /api/devices` - Update existing device
- `DELETE /api/devices` - Delete device
- `GET /api/devices/{id}` - Get device by ID
- `GET /api/devices/{id}/sensors` - Get sensors for device

### Sensors Management

- `GET /api/sensors` - List all sensors
- `POST /api/sensors` - Create new sensor
- `PUT /api/sensors` - Update existing sensor
- `DELETE /api/sensors` - Delete sensor
- `GET /api/sensors/{id}` - Get sensor by ID

### Measurements

- `POST /` - Direct measurement ingestion (single/batch)
- `POST /api/measurements` - Store measurements (single/batch)
- `GET /api/measurements` - Get all measurements
- `GET /api/measurements/latest` - Get latest measurement globally
- `GET /api/measurements/latest/all` - Get latest from all device-sensor pairs
- `GET /api/measurements/count` - Get total measurement count

### Device-Specific Measurements

- `GET /api/devices/{id}/measurements` - Get all measurements for device
- `GET /api/devices/{id}/sensors/{sensor_id}/measurements` - Get measurements
  for device-sensor pair
- `GET /api/devices/{id}/sensors/{sensor_id}/measurements/latest` - Get latest
  measurement
- `GET /api/devices/{id}/sensors/{sensor_id}/measurements/stats` - Get
  statistical summary

### System

- `GET /status/ping` - Health check endpoint
- `GET /metrics` - Prometheus metrics endpoint

## Background Tasks

### Measurement Ingestion

- Asynchronous processing via Tokio channel (8K buffer)
- Background thread handles database insertion
- Prevents blocking HTTP responses during high load

### View Maintenance

- Periodic refresh of materialized views
- Ensures query performance for device-sensor relationships

### Metrics Collection

- Continuous update of Prometheus metrics
- Tracks system performance and request patterns

## Caching Strategy

### Measurement Cache

- **Type**: Moka Cache with (device_id, sensor_id) keys
- **TTL**: 60 seconds
- **Capacity**: 128 entries
- **Purpose**: Fast retrieval of recent measurements

### Cache Usage

- Latest measurements per device-sensor pair
- Automatic cache population on cache misses
- Background refresh maintains cache freshness

## Metrics and Monitoring

### Prometheus Integration

- Request performance histograms by method and URI
- Custom application metrics
- Endpoint: `/metrics`

### Logging

- Structured JSON logging with tracing
- Configurable log levels (trace, debug, info, warn, error)
- Request/response logging with performance metrics

## Configuration

### Command Line Options

- `--host`: Server bind address (default: 0.0.0.0:65534)
- `--db-url`: PostgreSQL connection URL
- `--log-level`: Logging verbosity (default: info)

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string

## Requirements

- Rust 1.70+
- PostgreSQL 12+
- just (for task automation)

## Development Setup

### Build

```bash
cargo build
```

### Run (Development)

```bash
cargo run
```

### Configuration Help

```bash
cargo run -- -h
```

### Database Setup

The application expects a PostgreSQL database with the schema defined in the
migrations directory. Run migrations to set up the required tables and views.

## Deployment

### Docker Support

- Multi-stage Dockerfile for optimized builds
- Docker Compose configuration for local development

### Production Considerations

- Horizontal scaling through stateless design
- Database connection pooling via SQLx
- Metrics collection for monitoring
- Background task management for data consistency

## Performance Characteristics

- **Ingestion**: Asynchronous processing handles high-throughput measurement
  streams
- **Querying**: Cached latest measurements with sub-millisecond response times
- **Storage**: Efficient PostgreSQL indexing on timestamp and foreign keys
- **Memory**: Bounded cache prevents memory leaks during extended operation

## Future Enhancements

- [ ] WebSocket support for real-time measurement streaming
- [ ] Advanced analytics and alerting capabilities
