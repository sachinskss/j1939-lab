#![forbid(unsafe_code)]

use j1939_core::{Frame, TickMillis, GLOBAL_ADDRESS, PGN_TP_CM};
use j1939_tp::{BamTransmitter, Reassembler};
use std::hint::black_box;
use std::time::Instant;

const SAMPLES: usize = 10_000;

fn main() {
    report("Reassembler::on_frame", measure_decode());
    report("BamTransmitter::poll", measure_encode());
}

fn measure_decode() -> Vec<u128> {
    let frame = Frame::new(
        PGN_TP_CM,
        33,
        GLOBAL_ADDRESS,
        [0x20, 100, 0, 15, 0xFF, 0, 0xEF, 0],
    );
    (0..SAMPLES)
        .map(|_| {
            let mut receiver = Reassembler::new();
            let start = Instant::now();
            black_box(receiver.on_frame(black_box(&frame), TickMillis(0)).unwrap());
            start.elapsed().as_nanos()
        })
        .collect()
}

fn measure_encode() -> Vec<u128> {
    let payload = [0x5A; 100];
    (0..SAMPLES)
        .map(|_| {
            let mut transmitter = BamTransmitter::new(0xEF00, 33, &payload).unwrap();
            let start = Instant::now();
            black_box(transmitter.poll(TickMillis(0)));
            start.elapsed().as_nanos()
        })
        .collect()
}

fn report(name: &str, mut samples: Vec<u128>) {
    samples.sort_unstable();
    let percentile = |p: usize| samples[((samples.len() - 1) * p) / 100];
    println!(
        "{name} (microseconds, {SAMPLES} iterations): p50={:.3} p95={:.3} p99={:.3} max={:.3}",
        percentile(50) as f64 / 1000.0,
        percentile(95) as f64 / 1000.0,
        percentile(99) as f64 / 1000.0,
        samples[SAMPLES - 1] as f64 / 1000.0
    );
}
