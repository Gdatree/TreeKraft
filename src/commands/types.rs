use crate::main::vm::{VM, Value};
use std::collections::HashMap;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // ~struct Name(field1, field2)
    if (code.starts_with("~struct ") || code.starts_with("struct ")) && code.contains('(') && code.ends_with(')') {
        let rest = if code.starts_with("~struct ") { &code[8..] } else { &code[7..] };
        let (name, fields_s) = rest.split_once('(')?;
        let name = name.trim().to_string();
        let fields: Vec<String> = VM::split_top_level(&fields_s[..fields_s.len()-1], ',').into_iter().map(|f| f.trim().to_string()).collect();
        vm.structs.insert(name.clone(), fields.clone());
        vm.echo(&format!("struct {} defined", name));
        return Some(Ok(Value::None));
    }
    // ~enum Name(value1, value2)
    if (code.starts_with("~enum ") || code.starts_with("enum ")) && code.contains('(') && code.ends_with(')') {
        let rest = if code.starts_with("~enum ") { &code[6..] } else { &code[5..] };
        let (name, values_s) = rest.split_once('(')?;
        let name = name.trim().to_string();
        let values: Vec<String> = VM::split_top_level(&values_s[..values_s.len()-1], ',').into_iter().map(|v| v.trim().to_string()).collect();
        vm.enums.insert(name.clone(), values.clone());
        vm.echo(&format!("enum {} defined", name));
        return Some(Ok(Value::None));
    }
    // ~new("TypeName", args...) → создаёт экземпляр структуры
    if code.starts_with("~new(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len()-1], ',');
        if p.len() >= 1 {
            let type_name = VM::tq(&p[0]);
            if let Some(fields) = vm.structs.get(&type_name).cloned() {
                let mut dict = std::collections::HashMap::new();
                for (i, f) in fields.iter().enumerate() {
                    let val = if i + 1 < p.len() { vm.eval_value(&p[i+1]).unwrap_or(Value::None) } else { Value::None };
                    dict.insert(f.clone(), val);
                }
                let inst = Value::Dict(dict);
                vm.echo(inst.display());
                return Some(Ok(inst));
            }
            return Some(Err(format!("Unknown type '{}'", type_name)));
        }
        return None;
    }
    // ~match expr { pattern -> result ... } через case-стиль или match(x, v1, r1, v2, r2, ...)
    if code.starts_with("~match(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len()-1], ',');
        if p.len() >= 3 {
            let subj = vm.eval_value(&p[0]).unwrap_or(Value::None);
            let mut i = 1;
            while i + 1 < p.len() {
                let pat = vm.eval_value(&p[i]).unwrap_or(Value::None);
                if pat.display() == subj.display() || (pat.as_number().is_some() && pat.as_number() == subj.as_number()) {
                    let result = vm.eval_value(&p[i+1]).unwrap_or(Value::None);
                    vm.echo(&format!("{}", result.display()));
                    return Some(Ok(result));
                }
                i += 2;
            }
            // если пар нечётное число, последний элемент — default
            if p.len() % 2 == 0 {
                let default = vm.eval_value(&p[p.len()-1]).unwrap_or(Value::None);
                vm.echo(&format!("{}", default.display()));
                return Some(Ok(default));
            }
        }
        return None;
    }
    None
}
