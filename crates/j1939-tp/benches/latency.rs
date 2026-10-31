use criterion::{black_box, criterion_group, criterion_main, Criterion};
use j1939_core::TickMillis;
use j1939_tp::{BamTransmitter, Reassembler};

fn latency(c: &mut Criterion) {
    let payload = vec![0x5A; 100];
    c.bench_function("BamTransmitter::poll (encode path)", |b| {
        b.iter(|| {
            let mut tx = BamTransmitter::new(0xEF00, 33, black_box(&payload)).unwrap();
            black_box(tx.poll(TickMillis(0)))
        })
    });
    c.bench_function("Reassembler::on_frame (decode path)", |b| {
        let mut tx = BamTransmitter::new(0xEF00, 33, &payload).unwrap();
        let cm = tx.poll(TickMillis(0)).unwrap();
        b.iter(|| {
            let mut rx = Reassembler::new();
            black_box(rx.on_frame(black_box(&cm), TickMillis(0)).unwrap())
        })
    });
}

criterion_group!(benches, latency);
criterion_main!(benches);
