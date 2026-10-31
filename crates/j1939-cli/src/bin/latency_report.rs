#![forbid(unsafe_code)]

use j1939_core::{Frame, TickMillis, GLOBAL_ADDRESS, PGN_TP_CM};
use j1939_tp::{BamTransmitter, Reassembler};
use std::hint::black_box;
use std::time::Instant;

const SAMPLES: usize = 10_000;
const BATCH_SIZE: usize = 256;

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
            let mut receivers: Vec<_> = (0..BATCH_SIZE).map(|_| Reassembler::new()).collect();
            let start = Instant::now();
            for receiver in &mut receivers {
                black_box(receiver.on_frame(black_box(&frame), TickMillis(0)).unwrap());
            }
            start.elapsed().as_nanos() / BATCH_SIZE as u128
        })
        .collect()
}

fn measure_encode() -> Vec<u128> {
    let payload = [0x5A; 100];
    (0..SAMPLES)
        .map(|_| {
            let mut transmitters: Vec<_> = (0..BATCH_SIZE)
                .map(|_| BamTransmitter::new(0xEF00, 33, &payload).unwrap())
                .collect();
            let start = Instant::now();
            for transmitter in &mut transmitters {
                black_box(transmitter.poll(TickMillis(0)));
            }
            start.elapsed().as_nanos() / BATCH_SIZE as u128
        })
        .collect()
}

fn report(name: &str, mut samples: Vec<u128>) {
    samples.sort_unstable();
    let percentile = |p: usize| samples[((samples.len() - 1) * p) / 100];
    println!(
        "{name} (microseconds, {SAMPLES} samples, batches of {BATCH_SIZE}): p50={:.3} p95={:.3} p99={:.3} max={:.3}",
        percentile(50) as f64 / 1000.0,
        percentile(95) as f64 / 1000.0,
        percentile(99) as f64 / 1000.0,
        samples[SAMPLES - 1] as f64 / 1000.0
    );
}
