use crate::main::vm::{VM, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

thread_local! {
    static SRV: RefCell<Option<TcpListener>> = const { RefCell::new(None) };
    static ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static ROUTES: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~srv_start(") && code.ends_with(')') {
        let port = vm.eval_expr(&code[11..code.len()-1]).ok()? as u16;
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(listener) => { SRV.with(|s| *s.borrow_mut() = Some(listener)); println!("[Server] started on http://localhost:{}/", port); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~srv_stop") {
        SRV.with(|s| *s.borrow_mut() = None); println!("[Server] stopped"); Some(Ok(Value::None))
    } else if code.starts_with("~srv_root(") && code.ends_with(')') {
        let p = VM::tq(&code[10..code.len()-1]);
        let root = PathBuf::from(&p);
        ROOT.with(|r| *r.borrow_mut() = Some(root.clone()));
        println!("[Server] root: {}", root.display());
        Some(Ok(Value::None))
    } else if code.starts_with("~srv_route(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let path: String = VM::tq(&p[0]).trim().to_string();
            let body = match vm.eval_value(&p[1]) { Ok(v) => v.as_text().unwrap_or_default(), Err(e) => return Some(Err(e)) };
            ROUTES.with(|r| { let mut m = r.borrow_mut(); if m.is_none() { *m = Some(HashMap::new()); } m.as_mut().unwrap().insert(path.clone(), body); });
            println!("[Server] route: {}", path);
            return Some(Ok(Value::None));
        }
        Some(Err("~srv_route(path, body) — нужны 2 аргумента".to_string()))
    } else if code.starts_with("~srv_accept(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        let timeout_ms = if p.len() >= 1 { vm.eval_expr(&p[0]).ok()? as u64 } else { 1000 };
        let start = std::time::Instant::now();
        loop {
            let accepted = SRV.with(|s| {
                let mut s = s.borrow_mut();
                s.as_mut().and_then(|listener| {
                    listener.set_nonblocking(true).ok();
                    listener.accept().ok().map(|(stream, addr)| {
                        let peer = addr.to_string();
                        let mut buf = [0u8; 8192];
                        let n = stream.try_clone().and_then(|mut s2| s2.read(&mut buf)).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buf[..n]).to_string();
                        let first_line = request.lines().next().unwrap_or("").to_string();
                        let parts: Vec<&str> = first_line.split(' ').collect();
                        let method = parts.first().unwrap_or(&"").to_string();
                        let path = parts.get(1).unwrap_or(&"").to_string();
                        let body_start = request.find("\r\n\r\n").map(|i| i+4).unwrap_or(request.len());
                        let body = request[body_start..].to_string();
                        (peer, method, path, body, stream)
                    })
                })
            });
            if let Some((addr, method, path, body, mut stream)) = accepted {
                vm.set_var("_srv_addr", Value::Text(addr));
                vm.set_var("_srv_method", Value::Text(method.clone()));
                vm.set_var("_srv_path", Value::Text(path.clone()));
                vm.set_var("_srv_body", Value::Text(body));
                let resp = serve_request(&path);
                stream.write_all(&resp).ok();
                println!("[Server] {} {}", method, path);
                return Some(Ok(Value::Bool(true)));
            }
            if start.elapsed().as_millis() > timeout_ms as u128 { return Some(Ok(Value::Bool(false))); }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    } else {
        None
    }
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = |b: u8| -> Option<u8> {
                match b {
                    b'0'..=b'9' => Some(b - b'0'),
                    b'a'..=b'f' => Some(b - b'a' + 10),
                    b'A'..=b'F' => Some(b - b'A' + 10),
                    _ => None,
                }
            };
            if let (Some(a), Some(b)) = (h(bytes[i+1]), h(bytes[i+2])) {
                out.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn mime_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "txt" | "md" | "log" => "text/plain; charset=utf-8",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        _ => "application/octet-stream",
    }
}

fn serve_request(path: &str) -> Vec<u8> {
    let raw = url_decode(path.split('?').next().unwrap_or("/"));

    // Сначала проверяем ручные маршруты, поскольку ответ через srv_route важнее файла
    if let Some(body) = ROUTES.with(|r| r.borrow().as_ref().and_then(|m| m.get(&raw).cloned())) {
        let ct = if body.trim_start().starts_with('<') { "text/html; charset=utf-8" } else { "text/plain; charset=utf-8" };
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            ct, body.len()
        );
        let mut out = head.into_bytes();
        out.extend_from_slice(body.as_bytes());
        return out;
    }

    // Затем отдаём файлы из папки сервера, чтобы обычный сайт работал без скрипта
    let rel = raw.strip_prefix('/').unwrap_or(&raw);
    let rel = if rel.is_empty() { "index.html" } else { rel };
    if rel.split(['\\', '/']).any(|p| p == "..") {
        return "HTTP/1.1 403 Forbidden\r\nContent-Length: 9\r\nConnection: close\r\n\r\nForbidden".as_bytes().to_vec();
    }
    let root = ROOT.with(|r| r.borrow().clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let mut file_path = root.join(rel);
    // Неизвестный путь отдаём как index.html, чтобы одностраничник не ломался при обновлении
    if !file_path.is_file() {
        file_path = root.join("index.html");
    }
    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let mime = mime_for(&file_path.to_string_lossy());
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                mime, bytes.len()
            );
            let mut out = head.into_bytes();
            out.extend_from_slice(&bytes);
            out
        }
        Err(_) => "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found".as_bytes().to_vec(),
    }
}