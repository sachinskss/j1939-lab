These numbers show SDK overhead on a virtual bus. They do not show real CAN latency. Real CAN latency needs hardware.

Run `make bench` to run Criterion and print p50, p95, p99, and maximum core
operation latency over 10,000 samples. The percentile report times batches of
256 operations and divides by the batch size to reduce timer quantization. It
measures `Reassembler::on_frame` and `BamTransmitter::poll`. It does not include
bus, subprocess, Python scheduling, or hardware latency.

One local Windows run produced these sample values:

| Operation | p50 | p95 | p99 | Max |
|---|---:|---:|---:|---:|
| `Reassembler::on_frame` | 0.107 µs | 0.341 µs | 0.430 µs | 0.888 µs |
| `BamTransmitter::poll` | 0.009 µs | 0.009 µs | 0.010 µs | 0.079 µs |

These values describe one local run. Re-run the benchmark on the target machine
before comparing results.
