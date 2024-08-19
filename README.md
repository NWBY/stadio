# stadio

A simple reverse proxy written in Rust.

> [!WARNING]
> This is a work in progress and is not ready for use, right now I'd even suggest it's more of a proof of concept than anything else.

## Features

- Configurable via a YAML file
- Supports multiple backends
- Uses round robin load balancing

## Usage

1. Clone the repo
2. Copy `stadio.yaml.example` to `stadio.yaml` and edit it to include your backends
3. Run `cargo build`
4. Run `target/debug/stadio`
5. Visit `http://localhost:3001/` in your browser or curl it and see the response from your configured backends

## Todo

- Write tests
- Add support for weighted load balancing
- ~~Add basic logging~~
- Add basic metrics
- Benchmark
