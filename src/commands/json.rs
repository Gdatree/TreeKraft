use crate::main::vm::{VM, Value};

/// Модуль JSON: парсинг строк в значения и сериализация значений в JSON.
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // ПАРСИНГ
    if code.starts_with("~json_parse(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[12..code.len() - 1]) {
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
    } else if code.starts_with("~json_get(") && code.ends_with(')') {
        // json_get(obj, "path.to.field") -> значение по пути
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() >= 2 {
            let root = vm.eval_value(&p[0]).ok()?;
            let path = vm.eval_value(&p[1]).ok().and_then(|v| v.as_text()).unwrap_or_default();
            let mut cur = root;
            for key in path.split('.') {
                if key.is_empty() { continue; }
                match &cur {
                    Value::Dict(d) => {
                        if let Some(v) = d.get(key) {
                            cur = v.clone();
                        } else { return Some(Ok(Value::None)); }
                    }
                    Value::List(items) => {
                        if let Ok(i) = key.parse::<usize>() {
                            if i < items.len() { cur = items[i].clone(); } else { return Some(Ok(Value::None)); }
                        } else { return Some(Ok(Value::None)); }
                    }
                    _ => return Some(Ok(Value::None)),
                }
            }
            vm.echo(cur.display());
            Some(Ok(cur))
        } else { None }
    } else if code.starts_with("~json_keys(") && code.ends_with(')') {
        let root = match vm.eval_value(&code[11..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if let Value::Dict(d) = &root {
            let mut keys: Vec<&String> = d.keys().collect();
            keys.sort();
            let keys: Vec<Value> = keys.into_iter().map(|k| Value::Text(k.clone())).collect();
            let lv = Value::List(keys);
            vm.echo(lv.display()); Some(Ok(lv))
        } else {
            Some(Ok(Value::List(vec![])))
        }
    }
    // СЕРИАЛИЗАЦИЯ
    else if code.starts_with("~json_stringify(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[16..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let s = crate::main::vm::simple_json_stringify(&v);
        vm.echo(&s);
        Some(Ok(Value::Text(s)))
    } else if code.starts_with("~json_pretty(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[13..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let s = pretty(&v, 0);
        vm.echo(&s);
        Some(Ok(Value::Text(s)))
    } else {
        None
    }
}

/// Простая красивая печать JSON с отступами.
fn pretty(v: &Value, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let pad1 = "  ".repeat(indent + 1);
    match v {
        Value::Dict(d) => {
            if d.is_empty() { return "{}".to_string(); }
            let mut keys: Vec<&String> = d.keys().collect();
            keys.sort();
            let mut s = "{\n".to_string();
            for (i, k) in keys.iter().enumerate() {
                s.push_str(&pad1);
                s.push_str(&format!("\"{}\": {}", k, pretty(&d[*k], indent + 1)));
                if i + 1 < keys.len() { s.push(','); }
                s.push('\n');
            }
            s.push_str(&pad);
            s.push('}');
            s
        }
        Value::List(items) => {
            if items.is_empty() { return "[]".to_string(); }
            let mut s = "[\n".to_string();
            for (i, val) in items.iter().enumerate() {
                s.push_str(&pad1);
                s.push_str(&pretty(val, indent + 1));
                if i + 1 < items.len() { s.push(','); }
                s.push('\n');
            }
            s.push_str(&pad);
            s.push(']');
            s
        }
        _ => crate::main::vm::simple_json_stringify(v),
    }
}
