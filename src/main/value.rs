use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    Text(String),
    Bool(bool),
    List(Vec<Value>),
    /// Словарь, HashMap, O(1) доступ к ключам (было Vec > O(n²) на больших словарях)
    Dict(HashMap<String, Value>),
    Lambda(Vec<String>, String, u64),
    None,
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => s.trim().parse().ok(),
            _ => None,
        }
    }
    pub fn as_text(&self) -> Option<String> {
        match self {
            Value::Text(s) => Some(s.clone()),
            Value::Number(n) => Some(Self::fmt_num(*n)),
            Value::Bool(b) => Some(b.to_string()),
            Value::None => Some(String::new()),
            _ => None,
        }
    }
    pub fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::List(v) => !v.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::Lambda(..) => true,
            Value::None => false,
        }
    }
    pub fn fmt_num(n: f64) -> String {
        let s = format!("{}", n);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
    pub fn display(&self) -> String {
        match self {
            Value::Number(n) => Self::fmt_num(*n),
            Value::Text(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::None => "none".to_string(),
            Value::List(v) => format!(
                "[{}]",
                v.iter().map(|x| x.display()).collect::<Vec<_>>().join(", ")
            ),
            Value::Dict(d) => {
                // детерминированный порядок по ключу для стабильного вывода
                let mut keys: Vec<&String> = d.keys().collect();
                keys.sort();
                format!(
                    "{{{}}}",
                    keys.iter()
                        .map(|k| format!("{}: {}", k, d[*k].display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::Lambda(p, b, _) => format!("fn({})=>{}", p.join(","), b),
        }
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Text(_) => "text",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Lambda(..) => "lambda",
            Value::None => "none",
        }
    }
}