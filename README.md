# j1939-lab

A small, deterministic Rust demonstration of J1939 frame types and BAM transport,
with an in-memory bus, a command-line demo, and an async Python subprocess SDK.
The core crates do not perform I/O and are compiled without `std`.

## Requirements

- Rust 1.82.0 (pinned in `rust-toolchain.toml`)
- Python 3.10 or newer
- `make`

## Run it

```sh
make test
make lint
make demo
```

`make demo` transfers a 100-byte payload across the virtual bus and prints the
roundtrip result. `make test` installs the Python test extra, then runs Rust and
Python tests. `make bench` runs Criterion and reports p50, p95, p99, and max in
microseconds. See [docs/latency.md](docs/latency.md) for the scope of those numbers.

## Workspace

- `j1939-core`: `no_std` frame, PGN, NAME, and clock-value types.
- `j1939-tp`: `no_std` BAM transmitter and reassembler.
- `j1939-sim`: in-memory broadcast bus.
- `j1939-cli`: `bam_demo` executable and line-delimited `v1` subprocess protocol.
- `python/j1939_sdk`: async SDK; `python/pytest_j1939`: generic test fixtures.

## Protocol boundary

The subprocess accepts one JSON object per line on stdin and returns one JSON
object per line on stdout. Requests use `version: "v1"` and
`cmd: "send_bam"`; logs and diagnostics go to stderr. The Python test plugin
uses generic device, bus, and fault concepts.

## Scope and verification

RTS/CTS and address claiming are design notes only. The verification cases and
their status are in [docs/verification-report.md](docs/verification-report.md).
The project does not claim hardware timing or real CAN latency.
