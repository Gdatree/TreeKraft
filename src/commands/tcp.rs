use crate::main::vm::{VM, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

thread_local! {
    static TCP_RX: RefCell<HashMap<String, mpsc::Receiver<Vec<u8>>>> = RefCell::new(HashMap::new());
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~tcp_listen(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 1 {
            let port = vm.eval_value(&p[0]).ok()?.as_number()? as u16;
            let host = if p.len() >= 2 { vm.eval_value(&p[1]).ok()?.as_text().unwrap_or_else(|| "0.0.0.0".to_string()) } else { "0.0.0.0".to_string() };
            let addr = format!("{}:{}", host, port);
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    listener.set_nonblocking(true).ok();
                    let key = format!("tcp_server_{}", port);
                    vm.stacks.insert(key, vec![]);
                    thread::spawn(move || {
                        for stream in listener.incoming() {
                            match stream {
                                Ok(mut s) => {
                                    let mut buf = vec![0u8; 4096];
                                    match s.read(&mut buf) {
                                        Ok(n) => { buf.truncate(n); let _ = s.write_all(&buf); }
                                        Err(_) => {}
                                    }
                                }
                                Err(_) => { thread::sleep(std::time::Duration::from_millis(10)); }
                            }
                        }
                    });
                    vm.echo(&format!("TCP server on {}", addr));
                    Some(Ok(Value::Bool(true)))
                }
                Err(e) => Some(Err(e.to_string()))
            }
        } else { None }
    } else if code.starts_with("~tcp_connect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() >= 2 {
            let host = vm.eval_value(&p[0]).ok()?.as_text()?;
            let port = vm.eval_value(&p[1]).ok()?.as_number()? as u16;
            let name = if p.len() >= 3 { vm.eval_value(&p[2]).ok()?.as_text().unwrap_or_else(|| "default".to_string()) } else { "default".to_string() };
            let addr = format!("{}:{}", host, port);
            match TcpStream::connect(&addr) {
                Ok(mut stream) => {
                    stream.set_nonblocking(true).ok();
                    let (tx, rx) = mpsc::channel::<Vec<u8>>();
                    let key = format!("tcp_conn_{}", name);
                    vm.stacks.insert(key.clone(), vec![Value::Text(format!("connected to {}", addr))]);
                    TCP_RX.with(|m| { m.borrow_mut().insert(name.clone(), rx); });
                    vm.timers.insert(format!("tcp_rx_{}", name), std::time::Instant::now());
                    thread::spawn(move || {
                        let mut buf = vec![0u8; 4096];
                        loop {
                            match stream.read(&mut buf) {
                                Ok(0) | Err(_) => { thread::sleep(std::time::Duration::from_millis(50)); }
                                Ok(n) => {
                                    buf.truncate(n);
                                    if tx.send(buf.clone()).is_err() { break; }
                                    buf = vec![0u8; 4096];
                                }
                            }
                        }
                    });
                    Some(Ok(Value::Bool(true)))
                }
                Err(e) => Some(Err(e.to_string()))
            }
        } else { None }
    } else if code.starts_with("~tcp_send(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let name = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_else(|| "default".to_string());
            let data = vm.eval_value(&p[1]).ok()?.display();
            let key = format!("tcp_conn_{}", name);
            if let Some(Value::Text(ref s)) = vm.get_var(&key) {
                let addr = s.trim_start_matches("connected to ");
                let addr_full = format!("{}", addr);
                match TcpStream::connect(&addr_full) {
                    Ok(mut stream) => {
                        stream.write_all(data.as_bytes()).ok();
                        Some(Ok(Value::Bool(true)))
                    }
                    Err(e) => Some(Err(e.to_string()))
                }
            } else { Some(Err("Not connected".to_string())) }
        } else { None }
    } else if code.starts_with("~tcp_recv(") && code.ends_with(')') {
        let name = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text().unwrap_or_else(|| "default".to_string());
        // неблокирующее чтение из приёмника, заведённого в tcp_connect
        let got = TCP_RX.with(|m| {
            m.borrow().get(&name).and_then(|rx| rx.try_recv().ok())
        });
        match got {
            Some(bytes) => {
                let data = String::from_utf8_lossy(&bytes).to_string();
                vm.echo(&data);
                Some(Ok(Value::Text(data)))
            }
            None => Some(Ok(Value::None)),
        }
    } else {
        None
    }
}
