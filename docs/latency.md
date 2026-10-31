These numbers show SDK overhead on a virtual bus. They do not show real CAN latency. Real CAN latency needs hardware.

Run `make bench` to run Criterion and print p50, p95, p99, and maximum core
operation latency over 10,000 samples. The benchmark measures
`Reassembler::on_frame` and `BamTransmitter::poll`. It does not include bus,
subprocess, Python scheduling, or hardware latency.

One local Windows run produced these sample values:

| Operation | p50 | p95 | p99 | Max |
|---|---:|---:|---:|---:|
| `Reassembler::on_frame` | 0.200 µs | 0.300 µs | 0.400 µs | 11.700 µs |
| `BamTransmitter::poll` | 0.100 µs | 0.100 µs | 0.100 µs | 4.000 µs |

These values describe one local run. Re-run the benchmark on the target machine
before comparing results.
