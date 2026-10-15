use crate::main::vm::{VM, Value};
use std::collections::HashMap;
use std::fs;

pub fn save_state(vm: &VM, file: &str) -> Result<(), String> {
    let mut out = String::new();
    let mut pairs: Vec<_> = vm.variables.iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in pairs {
        out.push_str(k);
        out.push('=');
        out.push_str(&encode_value(v));
        out.push('\n');
    }
    fs::write(file, out).map_err(|e| e.to_string())
}

pub fn load_state(vm: &mut VM, file: &str) -> Result<(), String> {
    let text = fs::read_to_string(file).map_err(|e| e.to_string())?;
    vm.variables.clear();
    vm.depth_keys = vec![Vec::new()];
    vm.scope_depth = 0;
    vm.scopes = vec![HashMap::new()];
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, value)) = line.split_once('=') {
            if let Some(v) = decode_value(value.trim()) {
                vm.variables.insert(name.trim().to_string(), v);
            }
        }
    }
    Ok(())
}

pub fn eval_dcfg(vm: &mut VM, code: &str) -> Result<Value, String> {
    let parts = split_top(code, ' ');
    if parts.len() < 3 {
        return Err("~dcfg set/get/del/keys/vals/tovar".to_string());
    }
    match parts[1].as_str() {
        "set" => {
            if parts.len() < 5 {
                return Err("~dcfg set file key value".to_string());
            }
            let mut map = read_dtam(&parts[2]).unwrap_or_default();
            map.insert(parts[3].clone(), parts[4..].join(" "));
            write_dtam(&parts[2], &map)?;
            Ok(Value::None)
        }
        "get" => {
            if parts.len() != 4 {
                return Err("~dcfg get file key".to_string());
            }
            let map = read_dtam(&parts[2]).unwrap_or_default();
            if let Some(v) = map.get(&parts[3]) {
                println!("{}", v);
                Ok(Value::Text(v.clone()))
            } else {
                Err(format!("Key '{}' not found", parts[3]))
            }
        }
        "del" => {
            if parts.len() != 4 {
                return Err("~dcfg del file key".to_string());
            }
            let mut map = read_dtam(&parts[2]).unwrap_or_default();
            map.remove(&parts[3]);
            write_dtam(&parts[2], &map)?;
            Ok(Value::None)
        }
        "keys" => {
            let map = read_dtam(&parts[2]).unwrap_or_default();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let list: Vec<Value> = keys.into_iter().map(Value::Text).collect();
            println!("{}", Value::List(list.clone()).display());
            Ok(Value::List(list))
        }
        "vals" => {
            let map = read_dtam(&parts[2]).unwrap_or_default();
            let vals: Vec<Value> = map.values().cloned().map(Value::Text).collect();
            println!("{}", Value::List(vals.clone()).display());
            Ok(Value::List(vals))
        }
        "tovar" => {
            if parts.len() != 4 {
                return Err("~dcfg tovar file var".to_string());
            }
            let map = read_dtam(&parts[2]).unwrap_or_default();
            let items: Vec<Value> = map
                .into_iter()
                .map(|(k, v)| Value::List(vec![Value::Text(k), Value::Text(v)]))
                .collect();
            vm.set_var(&parts[3], Value::List(items.clone()));
            Ok(Value::List(items))
        }
        _ => Err("~dcfg set/get/del/keys/vals/tovar".to_string()),
    }
}

// Кодирование чтобы сохранять состояние между запусками

pub fn encode_value(v: &Value) -> String {
    match v {
        Value::Number(n) => {
            let s: String;
            if n.fract() == 0.0 && n.abs() < 1e15 {
                s = format!("{}", *n as i64);
            } else {
                s = Value::fmt_num(*n);
            }
            format!("N{}", if s.is_empty() { "0".to_string() } else { s })
        }
        Value::Text(s)   => format!("T{}", encode_text(s)),
        Value::Bool(b)   => format!("B{}", if *b { 1 } else { 0 }),
        Value::None      => "X".to_string(),
        Value::List(list) => {
            let mut out = String::from("L[");
            for (i, item) in list.iter().enumerate() {
                if i > 0 { out.push(','); }
                out.push_str(&encode_value(item));
            }
            out.push(']');
            out
        }
        Value::Dict(dict) => {
            let mut out = String::from("D{");
            let mut keys: Vec<&String> = dict.keys().collect();
            keys.sort();
            for (i, k) in keys.iter().enumerate() {
                if i > 0 { out.push(','); }
                out.push_str(&encode_text(k));
                out.push(':');
                out.push_str(&encode_value(&dict[*k]));
            }
            out.push('}');
            out
        }
        Value::Lambda(params, body, cap) => {
            format!("Λ{}|{}|{}", params.join(","), encode_text(body), cap)
        }
    }
}

pub fn decode_value(s: &str) -> Option<Value> {
    let t = s.trim();
    let mut chars = t.chars();
    let kind = chars.next()?;
    let rest = chars.as_str();
    match kind {
        'N' => rest.parse::<f64>().ok().map(Value::Number),
        'T' => Some(Value::Text(decode_text(rest))),
        'B' => Some(Value::Bool(rest == "1" || rest == "true")),
        'X' => Some(Value::None),
        'L' => {
            let r = rest.trim();
            if !r.starts_with('[') || !r.ends_with(']') { return None; }
            let inner = &r[1..r.len()-1];
            if inner.trim().is_empty() { return Some(Value::List(Vec::new())); }
            let mut list = Vec::new();
            for part in split_top(inner, ',') {
                list.push(decode_value(&part).unwrap_or(Value::Text(part)));
            }
            Some(Value::List(list))
        }
        'D' => {
            let r = rest.trim();
            if !r.starts_with('{') || !r.ends_with('}') { return None; }
            let inner = &r[1..r.len()-1];
            let mut dict = std::collections::HashMap::new();
            for part in split_top(inner, ',') {
                if let Some((k, v)) = part.split_once(':') {
                    dict.insert(
                        decode_text(k.trim()),
                        decode_value(v.trim()).unwrap_or(Value::None),
                    );
                }
            }
            Some(Value::Dict(dict))
        }
        'Λ' => {
            if let Some((params_str, rest)) = rest.split_once('|') {
                let params = if params_str.is_empty() {
                    vec![]
                } else {
                    params_str.split(',').map(|s| s.to_string()).collect()
                };
                let (body_enc, cap_s) = match rest.rsplit_once('|') {
                    Some((b, c)) => (b, c),
                    None => (rest, "0"),
                };
                let cap = cap_s.parse::<u64>().unwrap_or(0);
                Some(Value::Lambda(params, decode_text(body_enc), cap))
            } else {
                None
            }
        }
        _ => None,
    }
}

// Приватные помощники чтобы держать формат в одном месте

fn encode_text(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ','  => out.push_str("\\c"),
            '|'  => out.push_str("\\p"),
            ':'  => out.push_str("\\k"),
            '='  => out.push_str("\\e"),
            _    => out.push(ch),
        }
    }
    out
}

fn decode_text(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n')  => out.push('\n'),
                Some('r')  => out.push('\r'),
                Some('t')  => out.push('\t'),
                Some('c')  => out.push(','),
                Some('p')  => out.push('|'),
                Some('k')  => out.push(':'),
                Some('e')  => out.push('='),
                Some('\\') => out.push('\\'),
                Some(c)    => { out.push('\\'); out.push(c); }
                None       => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn split_top(text: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut q: Option<char> = None;
    let mut prev = '\0';
    for ch in text.chars() {
        if let Some(qc) = q {
            cur.push(ch);
            if ch == qc && prev != '\\' { q = None; }
            prev = ch;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => { q = Some(ch); cur.push(ch); }
            '(' | '[' | '{' => { depth += 1; cur.push(ch); }
            ')' | ']' | '}' => { depth -= 1; cur.push(ch); }
            _ if ch == delim && depth == 0 => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
        prev = ch;
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

fn read_dtam(file: &str) -> Result<HashMap<String, String>, String> {
    let text = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

fn write_dtam(file: &str, map: &HashMap<String, String>) -> Result<(), String> {
    let mut pairs: Vec<_> = map.iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    let mut out = String::new();
    for (k, v) in pairs {
        out.push_str(k);
        out.push_str(": ");
        out.push_str(v);
        out.push('\n');
    }
    fs::write(file, out).map_err(|e| e.to_string())
}