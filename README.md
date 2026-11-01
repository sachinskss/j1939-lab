# j1939-lab

[![CI](https://github.com/sachinskss/j1939-lab/actions/workflows/ci.yml/badge.svg)](https://github.com/sachinskss/j1939-lab/actions/workflows/ci.yml)

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

For fault scenarios, v1 requests may include `fault` with one of `drop`,
`reorder`, `abort`, `malformed`, or `duplicate`. `packet` selects a dropped
packet; `after_packets` and `reason` configure an abort. The subprocess returns
the same typed error kind produced by the reassembler.

## Scope and verification

RTS/CTS and address claiming are design notes only. CI generates a
scenario-by-layer verification report and status files from named Rust tests
and pytest JUnit XML; download the `verification-evidence` artifact from the
workflow run. [docs/verification-report.md](docs/verification-report.md)
describes that evidence path. The project does not claim hardware timing or
real bus latency.
