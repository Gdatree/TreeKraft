use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~json_encode(") && code.ends_with(')') {
        let name = code[13..code.len()-1].trim();
        let val = vm.get_var(name).unwrap_or(Value::None);
        let r = crate::main::vm::simple_json_stringify(&val);
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~json_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match crate::main::vm::json_parse(&s) {
            Ok(v) => { vm.echo(v.display()); Some(Ok(v)) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~yaml_encode(") && code.ends_with(')') {
        let v = vm.eval_value(&code[13..code.len()-1]).ok()?;
        let y = tk_to_yaml(&v);
        match serde_yaml::to_string(&y) {
            Ok(s) => { let s = s.trim_end().to_string(); vm.echo(&s); Some(Ok(Value::Text(s))) }
            Err(e) => Some(Err(format!("yaml_encode: {}", e))),
        }
    } else if code.starts_with("~yaml_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match serde_yaml::from_str::<serde_yaml::Value>(&s) {
            Ok(y) => { let v = yaml_to_tk(&y); vm.echo(v.display()); Some(Ok(v)) }
            Err(e) => Some(Err(format!("yaml_decode: {}", e))),
        }
    } else if code.starts_with("~toml_encode(") && code.ends_with(')') {
        let v = vm.eval_value(&code[13..code.len()-1]).ok()?;
        let t = tk_to_toml(&v);
        let tbl = match t {
            toml::Value::Table(m) => m,
            _ => return Some(Err("toml_encode needs dict (TOML top level is a table)".to_string())),
        };
        match toml::to_string(&toml::Value::Table(tbl)) {
            Ok(s) => { let s = s.trim_end().to_string(); vm.echo(&s); Some(Ok(Value::Text(s))) }
            Err(e) => Some(Err(format!("toml_encode: {}", e))),
        }
    } else if code.starts_with("~toml_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match s.parse::<toml::Value>() {
            Ok(t) => { let v = toml_to_tk(&t); vm.echo(v.display()); Some(Ok(v)) }
            Err(e) => Some(Err(format!("toml_decode: {}", e))),
        }
    } else if code.starts_with("~csv_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        let rows: Vec<Value> = s.lines().map(|line| Value::List(line.split(',').map(|c| Value::Text(c.trim().to_string())).collect())).collect();
        let r = Value::List(rows); vm.echo(r.display()); Some(Ok(r))
    } else if code.starts_with("~csv_encode(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r: String = list.iter().map(|row| match row { Value::List(cells) => cells.iter().map(|c| c.display()).collect::<Vec<_>>().join(","), other => other.display() }).collect::<Vec<_>>().join("\n");
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else {
        None
    }
}

/// Превращаем значение TK в узел YAML чтобы числа словари и списки сохранялись единообразно
fn tk_to_yaml(v: &Value) -> serde_yaml::Value {
    match v {
        Value::Number(n) => serde_yaml::Value::from(*n),
        Value::Text(s) => serde_yaml::Value::String(s.clone()),
        Value::Bool(b) => serde_yaml::Value::Bool(*b),
        Value::None => serde_yaml::Value::Null,
        Value::List(items) => serde_yaml::Value::Sequence(items.iter().map(tk_to_yaml).collect()),
        Value::Dict(d) => {
            let mut m = serde_yaml::Mapping::new();
            let mut keys: Vec<&String> = d.keys().collect();
            keys.sort();
            for k in keys {
                m.insert(serde_yaml::Value::String(k.clone()), tk_to_yaml(&d[k]));
            }
            serde_yaml::Value::Mapping(m)
        }
        Value::Lambda(p, b, _) => serde_yaml::Value::String(format!("fn({})=>{}", p.join(","), b)),
    }
}

/// Превращаем узел YAML обратно в значение TK чтобы скрипт получал привычные типы
fn yaml_to_tk(v: &serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::None,
        serde_yaml::Value::Bool(b) => Value::Bool(*b),
        serde_yaml::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_yaml::Value::String(s) => Value::Text(s.clone()),
        serde_yaml::Value::Sequence(items) => Value::List(items.iter().map(yaml_to_tk).collect()),
        serde_yaml::Value::Mapping(m) => {
            let mut d = std::collections::HashMap::new();
            for (k, v) in m {
                let ks = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{:?}", other),
                };
                d.insert(ks, yaml_to_tk(v));
            }
            Value::Dict(d)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_tk(&t.value),
    }
}

/// Превращаем значение TK в узел TOML где целые числа храним как целые если они влезают в границы i64
fn tk_to_toml(v: &Value) -> toml::Value {
    match v {
        Value::Number(n) => {
            if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                toml::Value::Integer(*n as i64)
            } else {
                toml::Value::Float(*n)
            }
        }
        Value::Text(s) => toml::Value::String(s.clone()),
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::None => toml::Value::String("none".to_string()),
        Value::List(items) => toml::Value::Array(items.iter().map(tk_to_toml).collect()),
        Value::Dict(d) => {
            let mut m = toml::map::Map::new();
            for (k, v) in d {
                m.insert(k.clone(), tk_to_toml(v));
            }
            toml::Value::Table(m)
        }
        Value::Lambda(p, b, _) => toml::Value::String(format!("fn({})=>{}", p.join(","), b)),
    }
}

/// Превращаем узел TOML обратно в значение TK где даты отдаем текстом чтобы не терять вид
fn toml_to_tk(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => Value::Text(s.clone()),
        toml::Value::Integer(n) => Value::Number(*n as f64),
        toml::Value::Float(n) => Value::Number(*n),
        toml::Value::Boolean(b) => Value::Bool(*b),
        toml::Value::Datetime(dt) => Value::Text(dt.to_string()),
        toml::Value::Array(items) => Value::List(items.iter().map(toml_to_tk).collect()),
        toml::Value::Table(m) => {
            let mut d = std::collections::HashMap::new();
            for (k, v) in m {
                d.insert(k.clone(), toml_to_tk(v));
            }
            Value::Dict(d)
        }
    }
}
