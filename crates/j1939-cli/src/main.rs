#![forbid(unsafe_code)]

use j1939_core::TickMillis;
use j1939_sim::VirtualBus;
use j1939_tp::{BamTransmitter, Message, Reassembler, TpError};
use std::io::{self, BufRead, Write};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("--stdio") => stdio_loop(),
        Some("run") => run_cli(),
        Some(arg) => Err(format!("unknown command: {arg}")),
        None => demo(),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn demo() -> Result<(), String> {
    let payload: Vec<u8> = (0..100).collect();
    let message = roundtrip(0xEF00, 33, payload)?;
    println!(
        "100-byte roundtrip: PGN {:#06x}, source {}, {} bytes",
        message.pgn,
        message.source,
        message.payload.len()
    );
    Ok(())
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(2).collect();
    let value = |key: &str, default: &str| -> Result<String, String> {
        Ok(args
            .windows(2)
            .find(|pair| pair[0] == key)
            .map(|pair| pair[1].clone())
            .unwrap_or_else(|| default.into()))
    };
    let pgn = value("--pgn", "61440")?
        .parse()
        .map_err(|_| "invalid --pgn".to_string())?;
    let source = value("--source", "33")?
        .parse()
        .map_err(|_| "invalid --source".to_string())?;
    let length: usize = value("--payload-len", "100")?
        .parse()
        .map_err(|_| "invalid --payload-len".to_string())?;
    if length > 1785 {
        return Err(TpError::PayloadTooLarge(length).to_string());
    }
    let message = roundtrip(pgn, source, (0..length).map(|n| n as u8).collect())?;
    println!(
        "roundtrip: PGN {:#x}, {} bytes",
        message.pgn,
        message.payload.len()
    );
    Ok(())
}

fn roundtrip(pgn: u32, source: u8, payload: Vec<u8>) -> Result<Message, String> {
    let mut tx = BamTransmitter::new(pgn, source, &payload).map_err(|e| e.to_string())?;
    let bus = VirtualBus::new();
    let receiver = bus.subscribe();
    let start = Instant::now();
    let task = thread::spawn(move || receive_until_message(receiver, start));
    while !tx.is_done() {
        let now = TickMillis(start.elapsed().as_millis() as u64);
        if let Some(frame) = tx.poll(now) {
            bus.broadcast(frame);
        }
        thread::sleep(Duration::from_millis(1));
    }
    task.join()
        .map_err(|_| "receiver thread panicked".to_string())?
}

fn receive_until_message(
    receiver: Receiver<j1939_core::Frame>,
    start: Instant,
) -> Result<Message, String> {
    let mut reassembler = Reassembler::new();
    loop {
        let frame = receiver
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| "receiver timed out".to_string())?;
        let now = TickMillis(start.elapsed().as_millis() as u64);
        if let Some(message) = reassembler
            .on_frame(&frame, now)
            .map_err(|e| e.to_string())?
        {
            return Ok(message);
        }
    }
}

fn stdio_loop() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let response = handle_request(&line);
        writeln!(stdout, "{response}")
            .and_then(|_| stdout.flush())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn handle_request(line: &str) -> String {
    if json_string(line, "version").as_deref() != Some("v1") {
        return error_json("ProtocolError", "unsupported protocol version");
    }
    if json_string(line, "cmd").as_deref() != Some("send_bam") {
        return error_json("MalformedRequest", "unsupported command");
    }
    let pgn = match json_number(line, "pgn") {
        Some(v) if v <= 0x3_FFFF => v as u32,
        _ => return error_json("MalformedRequest", "missing or invalid pgn"),
    };
    let source = match json_number(line, "source") {
        Some(v) if v <= 255 => v as u8,
        _ => return error_json("MalformedRequest", "missing or invalid source"),
    };
    let payload = match json_string(line, "payload") {
        Some(encoded) => match base64_decode(&encoded) {
            Some(payload) => payload,
            None => return error_json("MalformedRequest", "payload is not valid base64"),
        },
        None => Vec::new(),
    };
    if payload.len() > 1785 {
        return error_json(
            "PayloadTooLarge",
            &TpError::PayloadTooLarge(payload.len()).to_string(),
        );
    }
    match roundtrip(pgn, source, payload) {
        Ok(message) => format!("{{\"version\":\"v1\",\"ok\":true,\"message\":{{\"pgn\":{},\"source\":{},\"destination\":{},\"payload\":\"{}\"}}}}",
            message.pgn, message.source, message.destination, base64_encode(&message.payload)),
        Err(detail) => error_json("TransportError", &detail),
    }
}

fn error_json(kind: &str, detail: &str) -> String {
    let escaped = detail.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"version\":\"v1\",\"ok\":false,\"error\":{{\"kind\":\"{kind}\",\"detail\":\"{escaped}\"}}}}")
}

fn json_number(line: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{key}\"");
    let after = line
        .split_once(&marker)?
        .1
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn json_string(line: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\"");
    let after = line
        .split_once(&marker)?
        .1
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    let rest = after.strip_prefix('"')?;
    Some(rest.split('"').next()?.to_string())
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let clean = input.trim_end_matches('=');
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for ch in clean.bytes() {
        let value = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}
