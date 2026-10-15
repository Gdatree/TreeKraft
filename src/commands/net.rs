use crate::main::vm::{VM, Value};
use std::cell::RefCell;
use std::io::{Read, Write};

thread_local! {
    static UDP: RefCell<Option<std::net::UdpSocket>> = const { RefCell::new(None) };
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if VM::matches_cmd_prefix(code, "fetch(") || code.starts_with("~http_get(") && code.ends_with(')') {
        let url = if VM::matches_cmd_prefix(code, "fetch(") {
            let a = VM::extract_cmd_args(code, "fetch(")?;
            vm.eval_value(&a).ok()?.as_text()?
        } else {
            vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?
        };
        match http_get(&url) { Ok(r) => { vm.echo(&r); Some(Ok(Value::Text(r))) } Err(e) => Some(Err(e)) }
    } else if code.starts_with("~http_post(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let url = vm.eval_value(&p[0]).ok()?.as_text()?;
            let body = vm.eval_value(&p[1]).ok()?.display();
            let ct = if p.len() >= 3 { vm.eval_value(&p[2]).ok()?.as_text().unwrap_or_default() } else { "application/x-www-form-urlencoded".to_string() };
            match http_post(&url, &body, &ct) { Ok(r) => { vm.echo(&r); Some(Ok(Value::Text(r))) } Err(e) => Some(Err(e)) }
        } else { None }
    } else if code.starts_with("~http_put(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let url = vm.eval_value(&p[0]).ok()?.as_text()?;
            let body = vm.eval_value(&p[1]).ok()?.display();
            let ct = if p.len() >= 3 { vm.eval_value(&p[2]).ok()?.as_text().unwrap_or_default() } else { "application/x-www-form-urlencoded".to_string() };
            match http_method("PUT", &url, &body, &ct) { Ok(r) => { vm.echo(&r); Some(Ok(Value::Text(r))) } Err(e) => Some(Err(e)) }
        } else { None }
    } else if code.starts_with("~http_delete(") && code.ends_with(')') {
        let url = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match http_method("DELETE", &url, "", "application/json") { Ok(r) => { vm.echo(&r); Some(Ok(Value::Text(r))) } Err(e) => Some(Err(e)) }
    } else if code.starts_with("~ping(") && code.ends_with(')') {
        let host = vm.eval_value(&code[6..code.len()-1]).ok()?.as_text()?;
        match sys_ping(&host) {
            Ok(ms) => { vm.echo(Value::fmt_num(ms)); Some(Ok(Value::Number(ms))) }
            Err(e) => Some(Err(e)),
        }
    } else if code == "~ping" || code.starts_with("~ping(") {
        match sys_ping("8.8.8.8") {
            Ok(ms) => { vm.echo(Value::fmt_num(ms)); Some(Ok(Value::Number(ms))) }
            Err(e) => Some(Err(e)),
        }
    } else if code == "~online" || code == "online" || code == "~online()" {
        // Онлайном считаем живой пинг потому что ICMP проходит там где TCP часто режут прокси и фаерволы
        let ok = sys_ping("8.8.8.8").is_ok();
        vm.echo(ok); Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~ip") && (code.len() == 3 || code.ends_with(')')) {
        match local_ip() {
            Ok(ip) => { vm.echo(&ip); Some(Ok(Value::Text(ip))) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~dns_old(") && code.ends_with(')') {
        // Старое имя dns оставили чтобы прежние скрипты продолжали работать
        let host = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        match dns_lookup(&host) {
            Ok(addrs) => { vm.echo(&addrs); Some(Ok(Value::Text(addrs))) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~download(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() == 2 {
            let url = vm.eval_value(&p[0]).ok()?.as_text()?;
            let path = VM::tq(&p[1]);
            match http_get(&url) { Ok(data) => { std::fs::write(&path, &data).ok(); Some(Ok(Value::Bool(true))) } Err(e) => Some(Err(e)) }
        } else { None }
    } else if code.starts_with("~port_open(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 2 {
            let host = vm.eval_value(&p[0]).ok()?.as_text()?;
            let port = vm.eval_value(&p[1]).ok()?.as_number()? as u16;
            let addr: std::net::SocketAddr = match format!("{}:{}", host, port).parse() { Ok(a) => a, Err(_) => return Some(Err("Bad address".to_string())) };
            let ok = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).is_ok();
            vm.echo(ok); Some(Ok(Value::Bool(ok)))
        } else { None }
    } else if code == "~mac" || code == "~mac()" {
        match read_mac() {
            Ok(mac) => { vm.echo(&mac); Some(Ok(Value::Text(mac))) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~netstat") && (code.len() == 8 || code.ends_with(')')) {
        match std::process::Command::new("netstat").arg("-n").output() {
            Ok(o) => {
                let t = String::from_utf8_lossy(&o.stdout).to_string();
                vm.echo(&t);
                Some(Ok(Value::Text(t)))
            }
            Err(e) => Some(Err(format!("netstat failed: {}", e))),
        }
    } else if code.starts_with("~udp_bind(") && code.ends_with(')') {
        let port = vm.eval_value(&code[10..code.len()-1]).ok()?.as_number().unwrap_or(0.0) as u16;
        match std::net::UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(sock) => {
                sock.set_nonblocking(true).ok();
                UDP.with(|u| *u.borrow_mut() = Some(sock));
                vm.echo(&format!("UDP bound on port {}", port));
                Some(Ok(Value::Bool(true)))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~udp_send(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 3 {
            let host = vm.eval_value(&p[0]).ok()?.as_text()?;
            let port = vm.eval_value(&p[1]).ok()?.as_number()? as u16;
            let data = vm.eval_value(&p[2]).ok()?.display();
            let ok = UDP.with(|u| {
                let mut u = u.borrow_mut();
                if u.is_none() { *u = std::net::UdpSocket::bind("0.0.0.0:0").ok(); }
                u.as_mut().map(|s| s.send_to(data.as_bytes(), format!("{}:{}", host, port)).is_ok()).unwrap_or(false)
            });
            vm.echo(ok); Some(Ok(Value::Bool(ok)))
        } else { None }
    } else if code.starts_with("~udp_recv(") && code.ends_with(')') {
        let timeout_ms = vm.eval_value(&code[10..code.len()-1]).ok()?.as_number().unwrap_or(1000.0) as u64;
        let start = std::time::Instant::now();
        loop {
            let recv = UDP.with(|u| {
                let mut u = u.borrow_mut();
                if u.is_none() { *u = std::net::UdpSocket::bind("0.0.0.0:0").ok(); }
                let mut buf = [0u8; 65536];
                u.as_mut().and_then(|s| s.recv_from(&mut buf).ok().map(|(n, _)| String::from_utf8_lossy(&buf[..n]).to_string()))
            });
            if let Some(data) = recv { vm.echo(&data); return Some(Ok(Value::Text(data))); }
            if start.elapsed().as_millis() > timeout_ms as u128 { return Some(Ok(Value::None)); }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    } else if code == "~udp_close" || code.starts_with("~udp_close(") {
        UDP.with(|u| *u.borrow_mut() = None);
        vm.echo("UDP closed"); Some(Ok(Value::None))
    } else if code.starts_with("~dns(") && code.ends_with(')') {
        let host = vm.eval_value(&code[5..code.len()-1]).ok()?.as_text()?;
        match dns_lookup(&host) {
            Ok(addrs) => { vm.echo(&addrs); Some(Ok(Value::Text(addrs))) }
            Err(e) => Some(Err(e)),
        }
    } else {
        None
    }
}

/// DNS резолвим через систему чтобы подхватить ее настройки без лишнего кода
fn dns_lookup(host: &str) -> Result<String, String> {
    use std::net::ToSocketAddrs;
    let addrs: Vec<String> = format!("{}:80", host).to_socket_addrs()
        .map(|it| it.map(|a| a.ip().to_string()).collect())
        .unwrap_or_default();
    if addrs.is_empty() {
        Err(format!("DNS lookup failed: {}", host))
    } else {
        Ok(addrs.join(", "))
    }
}

/// Пинг делаем системной утилитой чтобы не тащить лишнюю работу с сокетами
/// Время ищем прямо в байтах чтобы ловить число даже при русской кодировке ответа
/// Обычный поиск по тексту там ломается поэтому читаем сырые байты а не строку
/// От маркера идем назад и собираем цифры чтобы получить задержку надежно
fn sys_ping(host: &str) -> Result<f64, String> {
    let out = std::process::Command::new("ping")
        .args(["-n", "1", "-w", "2000", host])
        .output()
        .map_err(|e| format!("ping failed: {}", e))?;
    if let Some(ms) = parse_ping_ms(&out.stdout) {
        return Ok(ms);
    }
    Err(format!("ping {} failed", host))
}

fn parse_ping_ms(raw: &[u8]) -> Option<f64> {
    let mut i = 0usize;
    while i + 1 < raw.len() {
        let is_ms = raw[i] == b'm' && raw[i + 1] == b's';
        let is_ru = raw[i] == 0xAC && raw[i + 1] == 0xE1; // Буквы мс в кодировке CP866 ищем байтами чтобы русский вывод тоже ловился
        if is_ms || is_ru {
            let mut j = i;
            while j > 0 && raw[j - 1].is_ascii_digit() { j -= 1; }
            if j < i {
                if let Ok(n) = std::str::from_utf8(&raw[j..i]).unwrap_or("").parse::<f64>() {
                    return Some(n);
                }
            }
        }
        i += 1;
    }
    None
}

/// Локальный IP узнаем без отправки трафика чтобы зря не трогать сеть
fn local_ip() -> Result<String, String> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    s.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    Ok(s.local_addr().map_err(|e| e.to_string())?.ip().to_string())
}

/// MAC читаем у первого адаптера чтобы отдать реальный адрес а не заглушку
fn read_mac() -> Result<String, String> {
    let out = std::process::Command::new("getmac")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .map_err(|_| "cannot read MAC address on this platform".to_string())?;
    let txt = String::from_utf8_lossy(&out.stdout);
    for line in txt.lines() {
        let first = line.split("\",\"").next().unwrap_or("").trim().trim_matches('"').trim();
        let parts: Vec<&str> = first.split('-').collect();
        if parts.len() == 6 && parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit())) {
            return Ok(first.to_string());
        }
    }
    Err("cannot read MAC address on this platform".to_string())
}

fn http_get(url: &str) -> Result<String, String> {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = if let Some(p) = url.find('/') { (&url[..p], &url[p..]) } else { (url, "/") };
    let addr_s = if host.contains(':') { host.to_string() } else { format!("{}:80", host) };
    let addr: std::net::SocketAddr = addr_s.parse().map_err(|e| format!("Bad addr: {}", e))?;
    let mut stream = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    let req = format!("GET {} HTTP/1.0\r\nHost: {}\r\nUser-Agent: TreeKraft/1.0.0\r\nConnection: close\r\n\r\n", path, host);
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp).map_err(|e| e.to_string())?;
    Ok(if let Some(pos) = resp.find("\r\n\r\n") { resp[pos+4..].to_string() } else { resp })
}

fn http_post(url: &str, body: &str, content_type: &str) -> Result<String, String> {
    http_method("POST", url, body, content_type)
}

fn http_method(method: &str, url: &str, body: &str, content_type: &str) -> Result<String, String> {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = if let Some(p) = url.find('/') { (&url[..p], &url[p..]) } else { (url, "/") };
    let addr_s = if host.contains(':') { host.to_string() } else { format!("{}:80", host) };
    let addr: std::net::SocketAddr = addr_s.parse().map_err(|e| format!("Bad addr: {}", e))?;
    let mut stream = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    let req = format!("{} {} HTTP/1.0\r\nHost: {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nUser-Agent: TreeKraft/1.0.0\r\nConnection: close\r\n\r\n{}",
        method, path, host, content_type, body.len(), body);
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp).map_err(|e| e.to_string())?;
    Ok(if let Some(pos) = resp.find("\r\n\r\n") { resp[pos+4..].to_string() } else { resp })
}
