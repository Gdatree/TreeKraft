use crate::main::vm::{VM, Value};
use flate2::write::{GzEncoder, GzDecoder};
use flate2::Compression;
use std::io::{Write, Read};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~gzip_encode(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[13..code.len()-1]) { Ok(v) => match v.as_text() { Some(t) => t, None => return Some(Err("Not text".to_string())) }, Err(e) => return Some(Err(e)) };
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        if enc.write_all(s.as_bytes()).is_err() { return Some(Err("Compression failed".to_string())); }
        let compressed = match enc.finish() { Ok(c) => c, Err(e) => return Some(Err(e.to_string())) };
        use base64::Engine;
        let r = base64::engine::general_purpose::STANDARD.encode(&compressed);
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~gzip_decode(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[13..code.len()-1]) { Ok(v) => match v.as_text() { Some(t) => t, None => return Some(Err("Not text".to_string())) }, Err(e) => return Some(Err(e)) };
        use base64::Engine;
        let bytes = match base64::engine::general_purpose::STANDARD.decode(&s) { Ok(b) => b, Err(e) => return Some(Err(e.to_string())) };
        let mut dec = GzDecoder::new(Vec::new());
        if dec.write_all(&bytes).is_err() { return Some(Err("Decompression failed".to_string())); }
        let r = match dec.finish() { Ok(data) => String::from_utf8_lossy(&data).to_string(), Err(e) => return Some(Err(e.to_string())) };
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~zip_list(") && code.ends_with(')') {
        let file = match vm.eval_value(&code[10..code.len()-1]) { Ok(v) => match v.as_text() { Some(t) => t, None => return Some(Err("Not text".to_string())) }, Err(e) => return Some(Err(e)) };
        match std::fs::File::open(&file) {
            Ok(f) => match zip::ZipArchive::new(f) {
                Ok(mut archive) => {
                    let names: Vec<Value> = (0..archive.len()).filter_map(|i| archive.by_index(i).ok().map(|e| Value::Text(e.name().to_string()))).collect();
                    let r = Value::List(names); vm.echo(r.display()); Some(Ok(r))
                }
                Err(e) => Some(Err(e.to_string())),
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~zip_extract(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() < 2 { return None; }
        let file = match vm.eval_value(&p[0]) { Ok(v) => match v.as_text() { Some(t) => t, None => return Some(Err("Not text".to_string())) }, Err(e) => return Some(Err(e)) };
        let name = match vm.eval_value(&p[1]) { Ok(v) => match v.as_text() { Some(t) => t, None => return Some(Err("Not text".to_string())) }, Err(e) => return Some(Err(e)) };
        match std::fs::File::open(&file) {
            Ok(f) => match zip::ZipArchive::new(f) {
                Ok(mut archive) => {
                    for i in 0..archive.len() {
                        if let Ok(mut e) = archive.by_index(i) {
                            if e.name() == name {
                                let mut data = Vec::new();
                                if e.read_to_end(&mut data).is_ok() {
                                    let r = String::from_utf8_lossy(&data).to_string();
                                    vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r)));
                                }
                            }
                        }
                    }
                    Some(Err("File not found in archive".to_string()))
                }
                Err(e) => Some(Err(e.to_string())),
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else {
        None
    }
}
