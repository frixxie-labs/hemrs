# hemrs (Home Environment Monitor-rs) 

This project is in need of a better name. Suggestions are welcome!

hemrs is a solution that retrieves and stores sensor data
It build using rust and docker

## Components

* Backend 
    - Contains a RESTful api over the database
* Collector
    - Firmware code for running Esp32 with a DHT11 sensor
* Sensor
    - Common datatypes between the Collector and Backend

## Requirements

* Rust
* Make
* Postgres instance

## How to build and run (in dev mode)

To build
```sh
cargo build
```

To run (with my defaults)
```sh
cargo run
```

For configuration options run
```sh
cargo run -- -h
```
