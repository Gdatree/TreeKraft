use crate::main::vm::{VM, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // ~ws_server(port, echo) — WebSocket-сервер (echo если true)
    if code.starts_with("~ws_server(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 1 {
            let port = vm.eval_value(&p[0]).ok()?.as_number()? as u16;
            let echo = p.len() >= 2 && vm.eval_cond(&p[1]).unwrap_or(true);
            let addr = format!("0.0.0.0:{}", port);
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    thread::spawn(move || {
                        for stream in listener.incoming() {
                            if let Ok(mut stream) = stream {
                                let echo = echo;
                                thread::spawn(move || {
                                    handle_ws_client(&mut stream, echo);
                                });
                            }
                        }
                    });
                    vm.echo(&format!("WebSocket server on ws://{}", addr));
                    return Some(Ok(Value::Bool(true)));
                }
                Err(e) => return Some(Err(e.to_string())),
            }
        }
        return None;
    }
    // ~ws_connect(host, port) — WebSocket-клиент (отправляет и получает одно сообщение)
    if code.starts_with("~ws_connect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 2 {
            let host = vm.eval_value(&p[0]).ok()?.as_text()?;
            let port = vm.eval_value(&p[1]).ok()?.as_number()? as u16;
            let msg = if p.len() >= 3 { vm.eval_value(&p[2]).ok()?.display() } else { "ping".to_string() };
            return match ws_client(&host, port, &msg) {
                Ok(resp) => { vm.echo(&resp); Some(Ok(Value::Text(resp))) }
                Err(e) => Some(Err(e)),
            };
        }
    }
    None
}

// SHA1 (чистый Rust)
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    let bit_len = (msg.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    let mut w = [0u32; 80];
    for chunk in msg.chunks(64) {
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(wi);
            e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d); h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, &v) in h.iter().enumerate() {
        out[i*4..i*4+4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn b64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

// WebSocket frames
fn encode_frame(data: &[u8], opcode: u8) -> Vec<u8> {
    let mut out = vec![0x80 | opcode];
    let len = data.len();
    if len < 126 {
        out.push(len as u8);
    } else if len < 65536 {
        out.push(126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(data);
    out
}

fn read_frame(stream: &mut TcpStream) -> Result<(u8, Vec<u8>), String> {
    let mut hdr = [0u8; 2];
    stream.read_exact(&mut hdr).map_err(|e| e.to_string())?;
    let fin_opcode = hdr[0];
    let opcode = fin_opcode & 0x0F;
    let masked = hdr[1] & 0x80 != 0;
    let mut len = (hdr[1] & 0x7F) as usize;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
        len = u16::from_be_bytes(ext) as usize;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
        len = u64::from_be_bytes(ext) as usize;
    }
    let mut mask = [0u8; 4];
    if masked {
        stream.read_exact(&mut mask).map_err(|e| e.to_string())?;
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).map_err(|e| e.to_string())?;
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }
    Ok((opcode, payload))
}

fn handle_ws_client(stream: &mut TcpStream, echo: bool) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) { Ok(n) => n, Err(_) => return };
    let req = String::from_utf8_lossy(&buf[..n]);
    let key = req.lines().find_map(|l| {
        let l = l.trim();
        l.strip_prefix("Sec-WebSocket-Key:").map(|k| k.trim().to_string())
    });
    if let Some(key) = key {
        let accept = b64(&sha1(format!("{}{}", key, WS_GUID).as_bytes()));
        let resp = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
            accept
        );
        if stream.write_all(resp.as_bytes()).is_err() { return; }
        let _ = stream.flush();
        // цикл чтения кадров
        loop {
            match read_frame(stream) {
                Ok((opcode, payload)) => {
                    match opcode {
                        0x8 => return, // close
                        0x9 => { let _ = stream.write_all(&encode_frame(&payload, 0xA)); } // pong
                        _ => {
                            if echo {
                                let mut mask = [0xAA, 0xBB, 0xCC, 0xDD];
                                let masked_payload: Vec<u8> = payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
                                let mut frame = vec![0x80 | 0x1, 0x80 | payload.len() as u8];
                                frame.extend_from_slice(&mask);
                                frame.extend_from_slice(&masked_payload);
                                if stream.write_all(&frame).is_err() { return; }
                            } else {
                                let _ = stream.write_all(&encode_frame(&payload, 0x1));
                            }
                        }
                    }
                }
                Err(_) => return,
            }
        }
    }
}

fn ws_client(host: &str, port: u16, msg: &str) -> Result<String, String> {
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let key_bytes: [u8; 16] = rng.gen();
    let key = b64(&key_bytes);
    let req = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
        addr, key
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
    let resp = String::from_utf8_lossy(&buf[..n]);
    if !resp.starts_with("HTTP/1.1 101") {
        return Err(format!("Handshake failed: {}", resp.lines().next().unwrap_or("")));
    }
    // отправить сообщение (masked)
    let mut mask = [0x55, 0x66, 0x77, 0x88];
    let masked_payload: Vec<u8> = msg.as_bytes().iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
    let mut frame = vec![0x80 | 0x1, 0x80 | msg.len() as u8];
    frame.extend_from_slice(&mask);
    frame.extend_from_slice(&masked_payload);
    stream.write_all(&frame).map_err(|e| e.to_string())?;
    // ждём ответ
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).ok();
        match read_frame(&mut stream) {
            Ok((_opcode, payload)) => {
                return Ok(String::from_utf8_lossy(&payload).to_string());
            }
            Err(_) => {}
        }
    }
    Err("Timeout waiting for response".to_string())
}
