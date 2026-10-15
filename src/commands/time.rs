use crate::main::vm::{VM, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Модуль времени, даты, файлов и сети: часы, таймеры, ping, файлы.
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Ниже работа с диском, чтобы скрипт мог читать, писать и смотреть папки

    if code.starts_with("~read(") && code.ends_with(')') {
        let path = VM::tq(&code[6..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(content) => {
                vm.echo(format!("{}", content));
                Some(Ok(Value::Text(content)))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~write(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let path = VM::tq(&p[0]);
            let content = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            match fs::write(&path, content) {
                Ok(_) => Some(Ok(Value::Bool(true))),
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~write(path, content)".to_string()))
        }
    } else if code.starts_with("~append(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let path = VM::tq(&p[0]);
            let content = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            let mut file = match fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(f) => f,
                Err(e) => return Some(Err(e.to_string())),
            };
            if let Err(e) = writeln!(file, "{}", content) {
                return Some(Err(e.to_string()));
            }
            Some(Ok(Value::Bool(true)))
        } else {
            Some(Err("~append(path, content)".to_string()))
        }
    } else if code.starts_with("~readlines(") && code.ends_with(')') {
        let path = VM::tq(&code[11..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<Value> = content
                    .lines()
                    .map(|l| Value::Text(l.to_string()))
                    .collect();
                let list = Value::List(lines.clone());
                vm.echo(format!("{}", list.display()));
                Some(Ok(list))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~dir(") && code.ends_with(')') {
        let path = VM::tq(&code[5..code.len() - 1]);
        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut items = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    items.push(Value::Text(name));
                }
                let list = Value::List(items.clone());
                vm.echo(format!("{}", list.display()));
                Some(Ok(list))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~mkdir(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len() - 1]);
        match fs::create_dir_all(&path) {
            Ok(_) => Some(Ok(Value::Bool(true))),
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~rmdir(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len() - 1]);
        let ok = fs::remove_dir_all(&path).is_ok();
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~delfile(") && code.ends_with(')') {
        let path = VM::tq(&code[9..code.len() - 1]);
        let ok = fs::remove_file(&path).is_ok();
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~cp(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[4..code.len() - 1], ',');
        if p.len() == 2 {
            let src = VM::tq(&p[0]);
            let dst = VM::tq(&p[1]);
            match fs::copy(&src, &dst) {
                Ok(_) => Some(Ok(Value::Bool(true))),
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~cp(src, dst)".to_string()))
        }
    } else if code.starts_with("~mv(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[4..code.len() - 1], ',');
        if p.len() == 2 {
            let src = VM::tq(&p[0]);
            let dst = VM::tq(&p[1]);
            match fs::rename(&src, &dst) {
                Ok(_) => Some(Ok(Value::Bool(true))),
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~mv(src, dst)".to_string()))
        }
    } else if code.starts_with("~rename(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let old = VM::tq(&p[0]);
            let new = VM::tq(&p[1]);
            match fs::rename(&old, &new) {
                Ok(_) => Some(Ok(Value::Bool(true))),
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~rename(old, new)".to_string()))
        }
    } else if code.starts_with("~fsize(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len() - 1]);
        match fs::metadata(&path) {
            Ok(meta) => {
                let size = meta.len();
                vm.echo(format!("{}", size));
                Some(Ok(Value::Number(size as f64)))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~ftype(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len() - 1]);
        match fs::metadata(&path) {
            Ok(meta) => {
                let typ = if meta.is_dir() { "dir" } else { "file" };
                vm.echo(format!("{}", typ));
                Some(Ok(Value::Text(typ.to_string())))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~ftouch(") && code.ends_with(')') {
        let path = VM::tq(&code[8..code.len() - 1]);
        if fs::metadata(&path).is_err() {
            if let Err(e) = fs::write(&path, "") {
                return Some(Err(e.to_string()));
            }
        }
        Some(Ok(Value::Bool(true)))
    } else if code.starts_with("~isdir(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len() - 1]);
        let ok = fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false);
        vm.echo(format!("{}", ok));
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~isfile(") && code.ends_with(')') {
        let path = VM::tq(&code[8..code.len() - 1]);
        let ok = fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false);
        vm.echo(format!("{}", ok));
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~modified(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len() - 1]);
        let secs = fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let s = secs_to_datetime(secs);
        vm.echo(format!("{}", s));
        Some(Ok(Value::Text(s)))
    } else if code.starts_with("~glob(") && code.ends_with(')') {
        let pattern = VM::tq(&code[6..code.len() - 1]);
        let mut matches = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if match_glob(&pattern, &name) {
                    matches.push(Value::Text(name));
                }
            }
        }
        let list = Value::List(matches.clone());
        vm.echo(format!("{}", list.display()));
        Some(Ok(list))
    } else if code.starts_with("~tempdir()") {
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        vm.echo(format!("{}", dir));
        Some(Ok(Value::Text(dir)))
    } else if code.starts_with("~readbytes(") && code.ends_with(')') {
        let path = VM::tq(&code[11..code.len() - 1]);
        match fs::read(&path) {
            Ok(bytes) => {
                let list: Vec<Value> = bytes.iter().map(|&b| Value::Number(b as f64)).collect();
                let list_val = Value::List(list.clone());
                vm.echo(format!("{} bytes", list.len()));
                Some(Ok(list_val))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~readcsv(") && code.ends_with(')') {
        let path = VM::tq(&code[9..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(text) => {
                let rows: Vec<Value> = text
                    .lines()
                    .map(|line| {
                        let cells: Vec<Value> = line
                            .split(',')
                            .map(|c| Value::Text(c.trim().to_string()))
                            .collect();
                        Value::List(cells)
                    })
                    .collect();
                let list = Value::List(rows.clone());
                vm.echo(format!("{} rows", rows.len()));
                Some(Ok(list))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~writecsv(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let path = VM::tq(&p[0]);
            let var = p[1].trim();
            if let Some(Value::List(rows)) = vm.get_var(var) {
                let mut out = String::new();
                for row in rows {
                    if let Value::List(cells) = row {
                        let line: String = cells
                            .iter()
                            .map(|c| c.display())
                            .collect::<Vec<_>>()
                            .join(",");
                        out.push_str(&line);
                        out.push('\n');
                    }
                }
                match fs::write(&path, out) {
                    Ok(_) => Some(Ok(Value::Bool(true))),
                    Err(e) => Some(Err(e.to_string())),
                }
            } else {
                Some(Err(format!("Variable '{}' is not a list", var)))
            }
        } else {
            Some(Err("~writecsv(path, list_var)".to_string()))
        }
    } else if code.starts_with("~readjson(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(text) => match crate::main::vm::json_parse(&text) {
                Ok(v) => {
                    vm.echo(format!("{}", v.display()));
                    Some(Ok(v))
                }
                Err(e) => Some(Err(e)),
            },
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~writejson(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let path = VM::tq(&p[0]);
            let var = p[1].trim();
            let v = vm.get_var(var).unwrap_or(Value::None);
            let json = crate::main::vm::simple_json_stringify(&v);
            match fs::write(&path, json) {
                Ok(_) => Some(Ok(Value::Bool(true))),
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~writejson(path, var)".to_string()))
        }
    } else if code.starts_with("~jsonparse(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[11..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        match crate::main::vm::json_parse(&s) {
            Ok(v) => {
                vm.echo(format!("{}", v.display()));
                Some(Ok(v))
            }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~jsonstringify(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[15..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let s = crate::main::vm::simple_json_stringify(&v);
        vm.echo(format!("{}", s));
        Some(Ok(Value::Text(s)))
    } else if code.starts_with("~save(") && code.ends_with(')') {
        let path = VM::tq(&code[6..code.len() - 1]);
        match crate::main::dtam::save_state(vm, &path) {
            Ok(_) => Some(Ok(Value::Bool(true))),
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~load(") && code.ends_with(')') {
        let path = VM::tq(&code[6..code.len() - 1]);
        match crate::main::dtam::load_state(vm, &path) {
            Ok(_) => Some(Ok(Value::Bool(true))),
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~dcfg ") {
        Some(crate::main::dtam::eval_dcfg(vm, code))
    } else if code.starts_with("~env(") && code.ends_with(')') {
        let key = VM::tq(&code[5..code.len() - 1]);
        let val = std::env::var(&key).unwrap_or_default();
        vm.echo(format!("{}", val));
        Some(Ok(Value::Text(val)))
    } else if code.starts_with("~setenv(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let key = VM::tq(&p[0]);
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            std::env::set_var(&key, val);
            Some(Ok(Value::Bool(true)))
        } else {
            Some(Err("~setenv(key, value)".to_string()))
        }
    } else if code.starts_with("~run(") && code.ends_with(')') {
        let cmd = VM::tq(&code[5..code.len() - 1]);
        Some(Ok(Value::Number(crate::main::vm::run_shell_status(&cmd) as f64)))
    } else if code.starts_with("~run ") {
        let cmd = code[5..].trim();
        let _ = crate::main::vm::run_shell_status(cmd);
        Some(Ok(Value::None))
    } else if code.starts_with("~shell(") && code.ends_with(')') {
        let cmd = VM::tq(&code[7..code.len() - 1]);
        let out = crate::main::vm::run_shell(&cmd);
        vm.echo(format!("{}", out));
        Some(Ok(Value::Text(out)))
    } else if code.starts_with("~execoutput(") && code.ends_with(')') {
        let cmd = VM::tq(&code[12..code.len() - 1]);
        let out = crate::main::vm::run_shell(&cmd);
        vm.echo(format!("{}", out));
        Some(Ok(Value::Text(out)))
    } else if code.starts_with("~exec(") && code.ends_with(')') {
        let cmd = VM::tq(&code[6..code.len() - 1]);
        let _ = crate::main::vm::run_shell_status(&cmd);
        Some(Ok(Value::None))
    } else if code.starts_with("~exit(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[6..code.len() - 1]).unwrap_or(0.0) as i32;
        std::process::exit(n);
    } else if code.starts_with("~clipboard()") {
        #[cfg(windows)]
        let out = crate::main::vm::run_shell("powershell -Command Get-Clipboard");
        #[cfg(not(windows))]
        let out = crate::main::vm::run_shell("xclip -selection clipboard -o 2>/dev/null || pbpaste 2>/dev/null");
        vm.echo(format!("{}", out));
        Some(Ok(Value::Text(out)))
    } else if code.starts_with("~setclipboard(") && code.ends_with(')') {
        let text = match vm.eval_value(&code[14..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        #[cfg(windows)]
        { let _ = crate::main::vm::run_shell(&format!("echo {}| clip", text)); }
        #[cfg(not(windows))]
        { let _ = crate::main::vm::run_shell(&format!("printf '%s' {} | xclip -selection clipboard", text.replace('\'', "'\\''"))); }
        Some(Ok(Value::Bool(true)))
    } else if code == "~os()" {
        let s = std::env::consts::OS;
        vm.echo(format!("{}", s));
        Some(Ok(Value::Text(s.to_string())))
    } else if code == "~arch()" {
        let s = std::env::consts::ARCH;
        vm.echo(format!("{}", s));
        Some(Ok(Value::Text(s.to_string())))
    } else if code == "~username()" {
        let u = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default();
        vm.echo(format!("{}", u));
        Some(Ok(Value::Text(u)))
    } else if code == "~hostname()" {
        let h = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| crate::main::vm::run_shell("hostname"));
        vm.echo(format!("{}", h));
        Some(Ok(Value::Text(h)))
    } else if code == "~homedir()" {
        let h = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default();
        vm.echo(format!("{}", h));
        Some(Ok(Value::Text(h)))
    } else if code == "~procs()" {
        #[cfg(windows)]
        let out = crate::main::vm::run_shell("tasklist");
        #[cfg(not(windows))]
        let out = crate::main::vm::run_shell("ps -eo pid,comm");
        vm.echo(format!("{}", out));
        Some(Ok(Value::Text(out)))
    } else if code.starts_with("~prockill(") && code.ends_with(')') {
        let pid = vm.eval_expr(&code[10..code.len() - 1]).unwrap_or(0.0) as u32;
        #[cfg(windows)]
        let _ = crate::main::vm::run_shell(&format!("taskkill /PID {} /F", pid));
        #[cfg(not(windows))]
        let _ = crate::main::vm::run_shell(&format!("kill -9 {}", pid));
        Some(Ok(Value::Bool(true)))
    } else if code.starts_with("~procexists(") && code.ends_with(')') {
        let pid = vm.eval_expr(&code[12..code.len() - 1]).unwrap_or(0.0) as u32;
        #[cfg(windows)]
        let out = crate::main::vm::run_shell(&format!("tasklist /FI \"PID eq {}\"", pid));
        #[cfg(not(windows))]
        let out = crate::main::vm::run_shell(&format!("ps -p {} -o pid=", pid));
        let ok = out.contains(&pid.to_string());
        vm.echo(format!("{}", ok));
        Some(Ok(Value::Bool(ok)))
    }

    // Ниже запросы в интернет, чтобы скрипт мог скачивать и отправлять данные

    else if code.starts_with("~fetch(") && code.ends_with(')') {
        let url = VM::tq(&code[7..code.len() - 1]);
        match crate::main::vm::simple_http_get(&url) {
            Ok(body) => {
                vm.echo(format!("{}", body));
                Some(Ok(Value::Text(body)))
            }
            Err(e) => Some(Err(format!("~fetch: {}", e))),
        }
    } else if code.starts_with("~download(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let url = VM::tq(&p[0]);
            let path = VM::tq(&p[1]);
            match crate::main::vm::simple_http_get(&url) {
                Ok(body) => {
                    match fs::write(&path, &body) {
                        Ok(_) => {
                            vm.echo(format!("Saved {} bytes to {}", body.len(), path));
                            Some(Ok(Value::Number(body.len() as f64)))
                        }
                        Err(e) => Some(Err(e.to_string())),
                    }
                }
                Err(e) => Some(Err(format!("~download: {}", e))),
            }
        } else {
            Some(Err("~download(url, file)".to_string()))
        }
    } else if code.starts_with("~post(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let url = VM::tq(&p[0]);
            let body = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            let url2 = url.trim_start_matches("http://").trim_start_matches("https://");
            let (host, path) = if let Some(pos) = url2.find('/') {
                (&url2[..pos], &url2[pos..])
            } else {
                (url2, "/")
            };
            let addr = format!("{}:80", host);
            match TcpStream::connect(&addr) {
                Ok(mut stream) => {
                    let req = format!(
                        "POST {} HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        path, host, body.len(), body
                    );
                    if let Err(e) = stream.write_all(req.as_bytes()) {
                        return Some(Err(e.to_string()));
                    }
                    let mut resp = String::new();
                    use std::io::Read;
                    if let Err(e) = stream.read_to_string(&mut resp) {
                        return Some(Err(e.to_string()));
                    }
                    let body_out = if let Some(pos) = resp.find("\r\n\r\n") {
                        resp[pos + 4..].to_string()
                    } else {
                        resp
                    };
                    vm.echo(format!("{}", body_out));
                    Some(Ok(Value::Text(body_out)))
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("~post(url, data)".to_string()))
        }
    } else if code.starts_with("~dns_lookup(") && code.ends_with(')') {
        let host = VM::tq(&code[12..code.len() - 1]);
        let out = crate::main::vm::run_shell(&format!("nslookup {}", host));
        vm.echo(&out);
        Some(Ok(Value::Text(out)))
    }

    // Ниже часы и таймеры, чтобы узнавать текущее время и мерить паузы

    else if code == "~now" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        vm.echo(secs);
        Some(Ok(Value::Text(secs.to_string())))
    } else if code == "~date" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let s = secs_to_datetime(secs);
        vm.echo(&s);
        Some(Ok(Value::Text(s)))
    } else if code == "~year()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (y, _, _) = ymd(secs);
        vm.echo(y);
        Some(Ok(Value::Number(y as f64)))
    } else if code == "~month()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (_, m, _) = ymd(secs);
        vm.echo(m);
        Some(Ok(Value::Number(m as f64)))
    } else if code == "~day()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (_, _, d) = ymd(secs);
        vm.echo(d);
        Some(Ok(Value::Number(d as f64)))
    } else if code == "~hour()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let h = (secs % 86400) / 3600;
        vm.echo(h);
        Some(Ok(Value::Number(h as f64)))
    } else if code == "~minute()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let m = (secs % 3600) / 60;
        vm.echo(m);
        Some(Ok(Value::Number(m as f64)))
    } else if code == "~second()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let s = secs % 60;
        vm.echo(s);
        Some(Ok(Value::Number(s as f64)))
    } else if code == "~weekday()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let d = (secs / 86400 + 4) % 7;
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        vm.echo(days[d as usize]);
        Some(Ok(Value::Text(days[d as usize].to_string())))
    } else if code == "~yearday()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (y, m, d) = ymd(secs);
        let months = if date_is_leap(y) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        let mut days = d;
        for i in 0..(m as usize - 1) {
            days += months[i];
        }
        vm.echo(days);
        Some(Ok(Value::Number(days as f64)))
    } else if code == "~pid()" || code == "~pid" {
        let pid = std::process::id();
        vm.echo(pid);
        Some(Ok(Value::Number(pid as f64)))
    } else if code == "~ppid()" {
        vm.echo("0");
        Some(Ok(Value::Number(0.0)))
    } else if code == "~timestamp()" || code == "~timestamp" {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        vm.echo(ms);
        Some(Ok(Value::Number(ms)))
    } else if code == "~nowms()" {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        vm.echo(ms);
        Some(Ok(Value::Number(ms)))
    } else if code == "~time()" {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let tod = secs % 86400;
        let s = format!("{:02}:{:02}:{:02}", tod / 3600, (tod % 3600) / 60, tod % 60);
        vm.echo(&s);
        Some(Ok(Value::Text(s)))
    } else if code == "~datetime()" {
        return resolve(vm, "~date");
    } else if code.starts_with("~sleepms(") && code.ends_with(')') {
        let ms = vm.eval_expr(&code[9..code.len() - 1]).unwrap_or(0.0) as u64;
        thread::sleep(Duration::from_millis(ms));
        Some(Ok(Value::None))
    } else if code.starts_with("~date_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let base = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let days = vm.eval_expr(&p[1]).unwrap_or(0.0) as u64;
            let new_ts = base + days * 86400;
            let s = secs_to_datetime(new_ts);
            vm.echo(format!("{}", s));
            Some(Ok(Value::Text(s)))
        } else {
            Some(Err("~date_add(offset, days)".to_string()))
        }
    } else if code.starts_with("~days_in_month(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 2 {
            let y = vm.eval_expr(&p[0]).unwrap_or(0.0) as u64;
            let m = vm.eval_expr(&p[1]).unwrap_or(0.0) as u64;
            let n = days_in_month(y, m);
            vm.echo(format!("{}", n));
            Some(Ok(Value::Number(n as f64)))
        } else {
            Some(Err("~days_in_month(year, month)".to_string()))
        }
    } else if code.starts_with("~is_leap(") && code.ends_with(')') {
        let y = vm.eval_expr(&code[9..code.len() - 1]).unwrap_or(0.0) as u64;
        let ok = date_is_leap(y);
        vm.echo(format!("{}", ok));
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~timer start") {
        let name = code[12..].trim();
        let name = if name.is_empty() { "_" } else { name };
        vm.timers.insert(name.to_string(), Instant::now());
        println!("Timer '{}' started", name);
        Some(Ok(Value::None))
    } else if code.starts_with("~timer stop") {
        let name = code[11..].trim().to_string();
        let n = if name.is_empty() { "_".to_string() } else { name };
        if let Some(start) = vm.timers.remove(&n) {
            let ms = start.elapsed().as_millis();
            vm.echo(format!("{}: {}ms", n, ms));
            Some(Ok(Value::Number(ms as f64)))
        } else {
            Some(Err(format!("Timer '{}' not started", n)))
        }
    } else if code.starts_with("~timer(") && code.ends_with(')') {
        let name = VM::tq(&code[7..code.len() - 1]);
        vm.timers.insert(name, Instant::now());
        Some(Ok(Value::None))
    } else if code.starts_with("~timer_elapsed(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len() - 1]);
        if let Some(start) = vm.timers.get(&name) {
            // Результат здесь не печатаем, так как внешняя печать иначе выведет его дважды
            let ms = start.elapsed().as_millis() as f64;
            Some(Ok(Value::Number(ms)))
        } else {
            Some(Err(format!("Timer '{}' not started", name)))
        }
    } else if code.starts_with("~autosave ") {
        let rest = code[10..].trim();
        if rest == "off" {
            vm.autosave_file = None;
            println!("Autosave off");
        } else {
            let f = if rest == "on" {
                "autosave.dtam".to_string()
            } else {
                rest.to_string()
            };
            println!("Autosave: {}", f);
            vm.autosave_file = Some(f);
        }
        Some(Ok(Value::None))
    }

    // Ниже очистка экрана, гудки и сведения о системе, чтобы управлять консолью

    else if code.starts_with("~filesize(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len()-1]); match fs::metadata(&path) { Ok(m) => { let s = m.len(); vm.echo(s); Some(Ok(Value::Number(s as f64))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~filetype(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len()-1]); match fs::metadata(&path) { Ok(m) => { let t = if m.is_dir() { "dir" } else { "file" }; vm.echo(format!("{}", t)); Some(Ok(Value::Text(t.to_string()))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~filetime(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len()-1]); match fs::metadata(&path) { Ok(m) => { if let Ok(t) = m.modified() { let s = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(); vm.echo(format!("{}", s)); Some(Ok(Value::Number(s as f64))) } else { Some(Err("No modified time".to_string())) } } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~copy(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len()-1], ','); if p.len() == 2 { let src = VM::tq(&p[0]); let dst = VM::tq(&p[1]); match fs::copy(&src, &dst) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) } } else { None }
    } else if code.starts_with("~rename(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ','); if p.len() == 2 { let src = VM::tq(&p[0]); let dst = VM::tq(&p[1]); match fs::rename(&src, &dst) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) } } else { None }
    } else if code.starts_with("~move(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len()-1], ','); if p.len() == 2 { let src = VM::tq(&p[0]); let dst = VM::tq(&p[1]); match fs::rename(&src, &dst) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) } } else { None }
    } else if code.starts_with("~remove_dir(") && code.ends_with(')') {
        let path = VM::tq(&code[12..code.len()-1]); match fs::remove_dir_all(&path) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~create_dir(") && code.ends_with(')') {
        let path = VM::tq(&code[12..code.len()-1]); match fs::create_dir_all(&path) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~list_dir(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len()-1]); match fs::read_dir(&path) { Ok(entries) => { let files: Vec<Value> = entries.filter_map(|e| e.ok()).map(|e| Value::Text(e.file_name().to_string_lossy().to_string())).collect(); let r = Value::List(files); vm.echo(format!("{}", r.display())); Some(Ok(r)) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~touch(") && code.ends_with(')') {
        let path = VM::tq(&code[7..code.len()-1]); match std::fs::OpenOptions::new().create(true).write(true).open(&path) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~env_get(") && code.ends_with(')') {
        let key = VM::tq(&code[9..code.len()-1]); let r = std::env::var(&key).unwrap_or_default(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~env_set(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() == 2 { let k = VM::tq(&p[0]); let v = VM::tq(&p[1]); std::env::set_var(k, v); Some(Ok(Value::Bool(true))) } else { None }
    } else if code.starts_with("~hostname") && (code.len() == 9 || (code.starts_with("~hostname(") && code.ends_with(')'))) {
        let r = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string()); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~username") && (code.len() == 9 || (code.starts_with("~username(") && code.ends_with(')'))) {
        let r = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string()); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~pid") && (code.len() == 4 || (code.starts_with("~pid(") && code.ends_with(')'))) {
        let r = std::process::id(); vm.echo(format!("{}", r)); Some(Ok(Value::Number(r as f64)))
    } else if code.starts_with("~cwd") && (code.len() == 4 || (code.starts_with("~cwd(") && code.ends_with(')'))) {
        let r = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~tempdir") && (code.len() == 8 || (code.starts_with("~tempdir(") && code.ends_with(')'))) {
        let r = std::env::temp_dir().to_string_lossy().to_string(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~homedir") && (code.len() == 8 || (code.starts_with("~homedir(") && code.ends_with(')'))) {
        let r = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~os") && (code.len() == 3 || (code.starts_with("~os(") && code.ends_with(')'))) {
        let r = std::env::consts::OS; vm.echo(format!("{}", r)); Some(Ok(Value::Text(r.to_string())))
    } else if code.starts_with("~arch") && (code.len() == 5 || (code.starts_with("~arch(") && code.ends_with(')'))) {
        let r = std::env::consts::ARCH; vm.echo(format!("{}", r)); Some(Ok(Value::Text(r.to_string())))
    } else if code.starts_with("~cpu_count") && (code.len() == 10 || (code.starts_with("~cpu_count(") && code.ends_with(')'))) {
        let r = std::thread::available_parallelism().map(|n| n.get() as f64).unwrap_or(1.0); vm.echo(format!("{}", Value::fmt_num(r))); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~read_file(") && code.ends_with(')') {
        let path = VM::tq(&code[11..code.len()-1]); match fs::read_to_string(&path) { Ok(c) => { vm.echo(format!("{}", c)); Some(Ok(Value::Text(c))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~write_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() == 2 { let path = VM::tq(&p[0]); let content = vm.eval_value(&p[1]).ok()?.as_text()?; match fs::write(&path, &content) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) } } else { None }
    } else if code.starts_with("~append_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() == 2 { let path = VM::tq(&p[0]); let content = vm.eval_value(&p[1]).ok()?.as_text()?; match fs::OpenOptions::new().append(true).create(true).open(&path) { Ok(mut f) => { use std::io::Write; f.write_all(content.as_bytes()).ok(); Some(Ok(Value::Bool(true))) } Err(e) => Some(Err(e.to_string())) } } else { None }
    } else if code == "~cls" || code == "~clear_screen()" {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
        Some(Ok(Value::None))
    } else if code == "~beep" || code == "~bell()" {
        print!("\x07");
        let _ = io::stdout().flush();
        Some(Ok(Value::None))
    } else if code.starts_with("~beep ") {
        let n = code[6..].trim().parse().unwrap_or(1usize);
        for _ in 0..n.min(20) {
            print!("\x07");
            let _ = io::stdout().flush();
            if n > 1 {
                thread::sleep(Duration::from_millis(200));
            }
        }
        Some(Ok(Value::None))
    } else if code.starts_with("~melody ") {
        for freq in code[8..].split_whitespace().filter_map(|s| s.parse::<u32>().ok()) {
            crate::main::platform::win_sound::beep(freq, 150);
            thread::sleep(Duration::from_millis(50));
        }
        Some(Ok(Value::None))
    } else if code.starts_with("~snd_beep(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let freq = vm.eval_expr(&p[0]).unwrap_or(440.0) as u32;
            let ms = vm.eval_expr(&p[1]).unwrap_or(100.0) as u32;
            crate::main::platform::win_sound::beep(freq, ms);
        }
        Some(Ok(Value::None))
    } else if code == "~sysinfo" || code == "~sys" {
        println!("OS: {} | Arch: {}", std::env::consts::OS, std::env::consts::ARCH);
        Some(Ok(Value::None))
    } else if code == "~version" {
        println!("TreeKraft u1");
        Some(Ok(Value::None))
    } else if code == "~version()" {
        println!("TreeKraft u1");
        Some(Ok(Value::None))
    } else if code == "~license()" {
        Some(Ok(Value::None))
    } else if code == "~exporthelp" || code == "~exporthelp()" {
        match fs::write("help.txt", "TreeKraft u1\nhttps://treekraft.tilda.ws/\n") {
            Ok(_) => {
                println!("Saved to help.txt");
                Some(Ok(Value::None))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code == "~args" {
        let list: Vec<Value> = vm.args.iter().map(|a| Value::Text(a.clone())).collect();
        let list_val = Value::List(list.clone());
        vm.echo(list_val.display());
        Some(Ok(list_val))
    } else if code == "~key()" || code == "kg.key()" {
        if let Some(ch) = crate::main::platform::win_key::key() {
            let s = ch.to_string();
            vm.echo(format!("{}", s));
            Some(Ok(Value::Text(s)))
        } else {
            Some(Ok(Value::Text(String::new())))
        }
    } else if code == "~vars" || code == "~vars()" {
        vm.print_vars_table();
        Some(Ok(Value::None))
    } else if code == "~trace on" {
        vm.trace = true;
        Some(Ok(Value::None))
    } else if code == "~trace off" {
        vm.trace = false;
        Some(Ok(Value::None))
    } else if code == "~history" {
        if vm.history.is_empty() {
            println!("  (empty)");
        } else {
            for (i, cmd) in vm.history.iter().enumerate() {
                println!("  {:>3}: {}", i + 1, cmd);
            }
        }
        Some(Ok(Value::None))
    } else if code == "~mem" {
        let vars: usize = vm.scopes.iter().map(|s| s.len()).sum();
        println!(
            "Variables: {}\nFunctions: {}\nStacks: {}\nQueues: {}\nSets: {}\nCache: {}\nSprites: {}\nAnimations: {}\nImages: {}\nTables: {}",
            vars,
            vm.functions.len(),
            vm.stacks.len(),
            vm.queues.len(),
            vm.sets.len(),
            vm.cache.len(),
            vm.sprites.len(),
            vm.animations.len(),
            vm.images.len(),
            vm.tables.len()
        );
        Some(Ok(Value::Number(vars as f64)))
    } else if code == "~memclear" {
        if let Some(g) = vm.scopes.first_mut() {
            g.clear();
        }
        vm.constants.clear();
        println!("Memory cleared");
        Some(Ok(Value::None))
    } else if code == "~reload" {
        *vm = VM::new();
        println!("TreeKraft reloaded");
        Some(Ok(Value::None))
    }

    // Ниже ввод с клавиатуры и мыши и паузы, чтобы скрипт мог ждать человека

    else if code.starts_with("~in(") && code.ends_with(')') {
        let prompt = match vm.eval_value(&code[4..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let s = input.trim_end_matches(['\r', '\n']).to_string();
                Some(Ok(Value::Text(s)))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code == "~waitkey()" || code == "~waitkey" {
        println!("Press any key...");
        let ch = crate::main::platform::win_key::getch();
        Some(Ok(Value::Text(ch.to_string())))
    } else if code == "~inkey()" || code == "~readkey()" {
        let ch = crate::main::platform::win_key::getch();
        vm.echo(format!("{}", ch));
        Some(Ok(Value::Text(ch.to_string())))
    } else if code.starts_with("~keydown(") && code.ends_with(')') {
        let want = VM::tq(&code[9..code.len() - 1]);
        let ok = crate::main::platform::win_key::key()
            .map(|c| c.to_string().eq_ignore_ascii_case(&want))
            .unwrap_or(false);
        vm.echo(format!("{}", ok));
        Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~keyup(") && code.ends_with(')') {
        vm.echo("false");
        Some(Ok(Value::Bool(false)))
    } else if code == "~keys()" {
        if let Some(c) = crate::main::platform::win_key::key() {
            let list = vec![Value::Text(c.to_string())];
            let list_val = Value::List(list.clone());
            vm.echo(format!("{}", list_val.display()));
            Some(Ok(list_val))
        } else {
            Some(Ok(Value::List(vec![])))
        }
    } else if code.starts_with("~keyname(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[9..code.len() - 1]).unwrap_or(0.0) as u8;
        let s = (n as char).to_string();
        vm.echo(format!("{}", s));
        Some(Ok(Value::Text(s)))
    } else if code.starts_with("~keycode(") && code.ends_with(')') {
        let s = VM::tq(&code[9..code.len() - 1]);
        let n = s.chars().next().map(|c| c as u32).unwrap_or(0);
        vm.echo(format!("{}", n));
        Some(Ok(Value::Number(n as f64)))
    } else if code == "~mouse()" {
        vm.echo("[0, 0]");
        Some(Ok(Value::List(vec![Value::Number(0.0), Value::Number(0.0)])))
    } else if code.starts_with("~mouse_btn(") && code.ends_with(')') {
        vm.echo("false");
        Some(Ok(Value::Bool(false)))
    } else if code == "~mouse_wheel()" {
        vm.echo("0");
        Some(Ok(Value::Number(0.0)))
    } else if code.starts_with("~waitms(") && code.ends_with(')') {
        let ms = vm.eval_expr(&code[8..code.len() - 1]).unwrap_or(0.0) as u64;
        thread::sleep(Duration::from_millis(ms));
        Some(Ok(Value::None))
    } else if code.starts_with("~waituntil(") && code.ends_with(')') {
        let cond = &code[11..code.len() - 1];
        let mut n = 0;
        while n < 10_000 {
            if vm.eval_cond(cond).unwrap_or(false) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
            n += 1;
        }
        Some(Ok(Value::None))
    } else if code.starts_with("~debounce(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let key = VM::tq(&p[0]);
            let ms = vm.eval_expr(&p[1]).unwrap_or(0.0) as u64;
            let now = Instant::now();
            let ok = match vm.debounce_last.get(&key) {
                Some(t) => now.duration_since(*t).as_millis() as u64 >= ms,
                None => true,
            };
            if ok {
                vm.debounce_last.insert(key, now);
            }
            vm.echo(format!("{}", ok));
            Some(Ok(Value::Bool(ok)))
        } else {
            Some(Err("~debounce(key, ms)".to_string()))
        }
    }

    else if code == "~shutdown" || code == "~shutdown()" { #[cfg(windows)] { std::process::Command::new("shutdown").args(["/s", "/t", "0"]).status().ok(); } #[cfg(not(windows))] { std::process::Command::new("shutdown").arg("-h").arg("now").status().ok(); } Some(Ok(Value::None)) }
    else if code == "~restart" || code == "~restart()" { #[cfg(windows)] { std::process::Command::new("shutdown").args(["/r", "/t", "0"]).status().ok(); } #[cfg(not(windows))] { std::process::Command::new("reboot").status().ok(); } Some(Ok(Value::None)) }
    else if code == "~logoff" || code == "~logoff()" { #[cfg(windows)] { std::process::Command::new("shutdown").args(["/l"]).status().ok(); } #[cfg(not(windows))] { std::process::Command::new("pkill").args(["-9", "-u"]).arg(std::env::var("USER").unwrap_or_default()).status().ok(); } Some(Ok(Value::None)) }
    else if code == "~sleep" || code == "~sleep()" { #[cfg(windows)] { std::process::Command::new("rundll32.exe").args(["powrprof.dll,SetSuspendState", "0,1,0"]).status().ok(); } #[cfg(not(windows))] { std::process::Command::new("systemctl").arg("suspend").status().ok(); } Some(Ok(Value::None)) }
    else if code.starts_with("~clipboard(") && code.ends_with(')') { Some(Ok(Value::Text(String::new()))) }
    else if code == "~uptime" || code == "~uptime()" { let u = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as f64; vm.echo(Value::fmt_num(u)); Some(Ok(Value::Number(u))) }
    else if code.starts_with("~year(") && code.ends_with(')') || code == "~year" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let y = &r[..4]; vm.echo(y); Some(Ok(Value::Text(y.to_string()))) }
    else if code.starts_with("~month(") && code.ends_with(')') || code == "~month" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let m = &r[5..7]; vm.echo(m); Some(Ok(Value::Text(m.to_string()))) }
    else if code.starts_with("~day(") && code.ends_with(')') || code == "~day" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let d = &r[8..10]; vm.echo(d); Some(Ok(Value::Text(d.to_string()))) }
    else if code.starts_with("~hour(") && code.ends_with(')') || code == "~hour" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let h = &r[11..13]; vm.echo(h); Some(Ok(Value::Text(h.to_string()))) }
    else if code.starts_with("~minute(") && code.ends_with(')') || code == "~minute" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let m = &r[14..16]; vm.echo(m); Some(Ok(Value::Text(m.to_string()))) }
    else if code.starts_with("~second(") && code.ends_with(')') || code == "~second" { let r = secs_to_datetime(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()); let s = &r[17..19]; vm.echo(s); Some(Ok(Value::Text(s.to_string()))) }
    else if code.starts_with("~weekday(") && code.ends_with(')') || code == "~weekday" { let days = ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"]; let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(); let wd = ((secs / 86400 + 4) % 7) as usize; vm.echo(days[wd]); Some(Ok(Value::Text(days[wd].to_string()))) }
    else if code.starts_with("~is_leap(") && code.ends_with(')') { let y = vm.eval_expr(&code[9..code.len()-1]).ok()? as u64; let r = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0); vm.echo(r); Some(Ok(Value::Bool(r))) }
    else if code.starts_with("~days_in_month(") && code.ends_with(')') { let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 2 { let y = vm.eval_expr(&p[0]).ok()? as u64; let m = vm.eval_expr(&p[1]).ok()? as u64; let r = days_in_month(y, m); vm.echo(r); Some(Ok(Value::Number(r as f64))) } else { None } }
    else if code.starts_with("~date_add(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 { let d = vm.eval_expr(&p[0]).ok()? as i64; let n = vm.eval_expr(&p[1]).ok()? as i64; vm.echo(d+n); Some(Ok(Value::Number((d+n) as f64))) } else { None } }
    else if code.starts_with("~sleepms(") && code.ends_with(')') { let ms = vm.eval_expr(&code[9..code.len()-1]).ok()? as u64; thread::sleep(Duration::from_millis(ms)); Some(Ok(Value::None)) }
    else if code.starts_with("~ppid(") && code.ends_with(')') || code == "~ppid" { vm.echo("0"); Some(Ok(Value::Number(0.0))) }
    else {
        None
    }
}

// Ниже помощники для дат, которые считают дни без внешних библиотек

fn secs_to_datetime(secs: u64) -> String {
    let (y, m, d) = ymd(secs);
    let tod = secs % 86400;
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, tod / 3600, (tod % 3600) / 60, tod % 60)
}

fn ymd(secs: u64) -> (u64, u64, u64) {
    let mut days = secs / 86400;
    let mut y = 1970u64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let din = if leap { 366 } else { 365 };
        if days < din {
            break;
        }
        days -= din;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let months = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for dm in months.iter() {
        if days < *dm {
            break;
        }
        days -= dm;
        m += 1;
    }
    (y, m, days + 1)
}

fn date_is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn days_in_month(y: u64, m: u64) -> u64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if date_is_leap(y) { 29 } else { 28 },
        _ => 30,
    }
}

fn match_glob(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    fn mg(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (None, _) => false,
            (Some(&'*'), _) => mg(&p[1..], t) || (!t.is_empty() && mg(p, &t[1..])),
            (_, None) => p.iter().all(|&c| c == '*'),
            (Some(&'?'), _) => mg(&p[1..], &t[1..]),
            (Some(pc), Some(tc)) => pc == tc && mg(&p[1..], &t[1..]),
        }
    }
    mg(&p, &t)
}